use std::future::Future;

use rustgram::Request;
use sentc_crypto_common::group::{CreateData, GroupCreateOutput, GroupDataCheckUpdateServerOutput};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::GroupId;
use server_core::cache;
use server_core::input_helper::{bytes_to_json, get_raw_body};
use server_core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};

use crate::customer_app::app_util::{check_endpoint_with_req, Endpoint};
use crate::file::file_service;
use crate::group::group_entities::{GroupServerData, GroupUserKeys, ListGroups};
use crate::group::{get_group_user_data_from_req, group_model};
use crate::user::jwt::get_jwt_data_from_param;
use crate::util::api_res::{echo, echo_success, ApiErrorCodes, HttpErr, JRes};
use crate::util::get_group_cache_key;

pub(crate) fn create(req: Request) -> impl Future<Output = JRes<GroupCreateOutput>>
{
	create_group(req, None, None)
}

pub(crate) async fn create_child_group(req: Request) -> JRes<GroupCreateOutput>
{
	//this is called in the group mw from the parent group id
	let group_data = get_group_user_data_from_req(&req)?;
	let parent_group_id = Some(group_data.group_data.id.to_string());
	let user_rank = Some(group_data.user_data.rank);

	create_group(req, parent_group_id, user_rank).await
}

async fn create_group(mut req: Request, parent_group_id: Option<GroupId>, user_rank: Option<i32>) -> JRes<GroupCreateOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupCreate)?;

	let user = get_jwt_data_from_param(&req)?;

	let input: CreateData = bytes_to_json(&body)?;

	let group_id = group_model::create(
		user.sub.to_string(),
		user.id.to_string(),
		input,
		parent_group_id,
		user_rank,
	)
	.await?;

	let out = GroupCreateOutput {
		group_id,
	};

	echo(out)
}

pub(crate) async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupDelete)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let children = group_model::delete(
		group_data.group_data.app_id.to_string(),
		group_data.group_data.id.to_string(),
		group_data.user_data.rank,
	)
	.await?;

	//children incl. the deleted group
	file_service::delete_file_for_group(
		group_data.group_data.app_id.as_str(),
		group_data.group_data.id.as_str(),
		children,
	)
	.await?;

	//don't delete cache for each group user, but for the group
	let key_group = get_group_cache_key(
		group_data.group_data.app_id.as_str(),
		group_data.group_data.id.as_str(),
	);
	cache::delete(key_group.as_str()).await;

	echo_success()
}

pub(crate) async fn get_user_group_data(req: Request) -> JRes<GroupServerData>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDataGet)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let app_id = &group_data.group_data.app_id;
	let group_id = &group_data.group_data.id;
	let user_id = &group_data.user_data.user_id;

	let keys = group_model::get_user_group_keys(
		app_id.to_string(),
		group_id.to_string(),
		user_id.to_string(),
		0, //fetch the first page
		"".to_string(),
	)
	.await?;

	let key_update = group_model::check_for_key_update(app_id.to_string(), user_id.to_string(), group_id.to_string()).await?;

	let parent = match &group_data.group_data.parent {
		Some(p) => Some(p.to_string()),
		None => None,
	};

	let out = GroupServerData {
		group_id: group_id.to_string(),
		parent_group_id: parent,
		keys,
		key_update,
		rank: group_data.user_data.rank,
		created_time: group_data.group_data.time,
		joined_time: group_data.user_data.joined_time,
	};

	echo(out)
}

/**
Check with this fn if:
- the user is still in the group (via mw)
- the rank of the user
- if there is a key update

This is used in the client, when the group data is cached in the client
and the data gets fetched from the cache.
*/
pub(crate) async fn get_key_update_for_user(req: Request) -> JRes<GroupDataCheckUpdateServerOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserUpdateCheck)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let key_update = group_model::check_for_key_update(
		group_data.group_data.app_id.to_string(),
		group_data.user_data.user_id.to_string(),
		group_data.group_data.id.to_string(),
	)
	.await?;

	let out = GroupDataCheckUpdateServerOutput {
		key_update,
		rank: group_data.user_data.rank,
	};

	echo(out)
}

pub(crate) async fn get_user_group_keys(req: Request) -> JRes<Vec<GroupUserKeys>>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserKeys)?;

	//don't get the group data from mw cache, this is done by the model fetch
	let group_data = get_group_user_data_from_req(&req)?;

	let params = get_params(&req)?;
	let last_k_id = get_name_param_from_params(&params, "last_k_id")?;
	let last_fetched_time = get_name_param_from_params(&params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let user_keys = group_model::get_user_group_keys(
		group_data.group_data.app_id.to_string(),
		group_data.group_data.id.to_string(),
		group_data.user_data.user_id.to_string(),
		last_fetched_time,
		last_k_id.to_string(),
	)
	.await?;

	echo(user_keys)
}

pub(crate) async fn get_user_group_key(req: Request) -> JRes<GroupUserKeys>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserKeys)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let key_id = get_name_param_from_req(&req, "key_id")?;

	let key = group_model::get_user_group_key(
		group_data.group_data.app_id.to_string(),
		group_data.group_data.id.to_string(),
		group_data.user_data.user_id.to_string(),
		key_id.to_string(),
	)
	.await?;

	echo(key)
}

pub(crate) async fn get_all_groups_for_user(req: Request) -> JRes<Vec<ListGroups>>
{
	//this is called from the user without a group id

	check_endpoint_with_req(&req, Endpoint::GroupList)?;

	let user = get_jwt_data_from_param(&req)?;
	let params = get_params(&req)?;
	let last_group_id = get_name_param_from_params(&params, "last_group_id")?;
	let last_fetched_time = get_name_param_from_params(&params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let list = group_model::get_all_groups_to_user(
		user.sub.to_string(),
		user.id.to_string(),
		last_fetched_time,
		last_group_id.to_string(),
	)
	.await?;

	echo(list)
}

use std::future::Future;

use rustgram::Request;
use sentc_crypto::sdk_common::GroupId;
use sentc_crypto_common::group::{CreateData, GroupCreateOutput, GroupDeleteServerOutput, GroupKeyServerOutput, GroupServerData};

use crate::core::api_res::{echo, ApiErrorCodes, HttpErr, JRes};
use crate::core::cache;
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::core::url_helper::{get_name_param_from_params, get_params};
use crate::customer_app::app_util::{check_endpoint_with_req, Endpoint};
use crate::group::{get_group_user_data_from_req, group_model};
use crate::user::jwt::get_jwt_data_from_param;
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

pub(crate) async fn delete(req: Request) -> JRes<GroupDeleteServerOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupDelete)?;

	let group_data = get_group_user_data_from_req(&req)?;

	group_model::delete(
		group_data.group_data.app_id.to_string(),
		group_data.group_data.id.to_string(),
		group_data.user_data.rank,
	)
	.await?;

	//don't delete cache for each group user, but for the group
	let key_group = get_group_cache_key(
		group_data.group_data.app_id.as_str(),
		group_data.group_data.id.as_str(),
	);
	cache::delete(key_group.as_str()).await;

	let out = GroupDeleteServerOutput {
		group_id: group_data.group_data.id.to_string(),
	};

	echo(out)
}

pub(crate) async fn get_user_group_data(req: Request) -> JRes<GroupServerData>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDataGet)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let app_id = &group_data.group_data.app_id;
	let group_id = &group_data.group_data.id;
	let user_id = &group_data.user_data.user_id;

	let user_keys = group_model::get_user_group_keys(
		app_id.to_string(),
		group_id.to_string(),
		user_id.to_string(),
		0, //fetch the first page
		"".to_string(),
	)
	.await?;

	let mut keys: Vec<GroupKeyServerOutput> = Vec::with_capacity(user_keys.len());

	for user_key in user_keys {
		keys.push(user_key.into());
	}

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

pub(crate) async fn get_user_group_keys(req: Request) -> JRes<Vec<GroupKeyServerOutput>>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDataGet)?;

	//don't get the group data from mw cache, this is done by the model fetch

	let user = get_jwt_data_from_param(&req)?;
	let req_params = get_params(&req)?;
	let group_id = get_name_param_from_params(req_params, "group_id")?;

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
		user.sub.to_string(),
		group_id.to_string(),
		user.id.to_string(),
		last_fetched_time,
		last_k_id.to_string(),
	)
	.await?;

	let mut keys: Vec<GroupKeyServerOutput> = Vec::with_capacity(user_keys.len());
	for user_key in user_keys {
		keys.push(user_key.into());
	}

	echo(keys)
}

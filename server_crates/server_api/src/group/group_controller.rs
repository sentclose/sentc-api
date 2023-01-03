use std::future::Future;

use rustgram::Request;
use sentc_crypto_common::group::{CreateData, GroupCreateOutput, GroupDataCheckUpdateServerOutput, GroupLightServerData};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::GroupId;
use server_core::cache;
use server_core::input_helper::{bytes_to_json, get_raw_body};
use server_core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};

use crate::customer_app::app_util::{check_endpoint_with_app_options, check_endpoint_with_req, get_app_data_from_req, Endpoint};
use crate::group::group_entities::{GroupChildrenList, GroupServerData, GroupUserKeys, ListGroups};
use crate::group::group_user_service::NewUserType;
use crate::group::{get_group_user_data_from_req, group_model, group_service, GROUP_TYPE_NORMAL};
use crate::user::jwt::get_jwt_data_from_param;
use crate::user::user_entities::UserPublicKeyDataEntity;
use crate::util::api_res::{echo, echo_success, ApiErrorCodes, HttpErr, JRes};
use crate::util::get_group_cache_key;

pub fn create(req: Request) -> impl Future<Output = JRes<GroupCreateOutput>>
{
	create_group(req, None, None, None, false)
}

pub async fn create_child_group(req: Request) -> JRes<GroupCreateOutput>
{
	//this is called in the group mw from the parent group id
	let group_data = get_group_user_data_from_req(&req)?;
	let parent_group_id = Some(group_data.group_data.id.to_string());
	let user_rank = Some(group_data.user_data.rank);

	//a connected group can also got children but these children will be a connected group too
	let is_connected_group = group_data.group_data.is_connected_group;

	create_group(req, parent_group_id, user_rank, None, is_connected_group).await
}

pub async fn create_connected_group_from_group(req: Request) -> JRes<GroupCreateOutput>
{
	/*
	- A connected group is a group where other groups can join or can get invited, not only users.
	- A connected group can also got children (which are marked as connected group too)
	- A connected group cannot be created from a already connected group.
		Because the users of the one connected group cannot access the connected group.
		So only non connected groups can create connected groups.

	- Users can join both groups
	 */

	//the same as parent group, but this time with the group as member, not as parent
	let group_data = get_group_user_data_from_req(&req)?;
	let connected_group_id = Some(group_data.group_data.id.to_string());
	let user_rank = Some(group_data.user_data.rank);

	if group_data.group_data.is_connected_group {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupConnectedFromConnected,
			"Can't create a connected group from a connected group".to_string(),
			None,
		));
	}

	create_group(req, None, user_rank, connected_group_id, true).await
}

async fn create_group(
	mut req: Request,
	parent_group_id: Option<GroupId>,
	user_rank: Option<i32>,
	connected_group: Option<GroupId>,
	is_connected_group: bool,
) -> JRes<GroupCreateOutput>
{
	let body = get_raw_body(&mut req).await?;

	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::GroupCreate)?;

	let user = get_jwt_data_from_param(&req)?;

	let input: CreateData = bytes_to_json(&body)?;

	let group_id = group_service::create_group(
		app.app_data.app_id.to_string(),
		user.id.to_string(),
		input,
		GROUP_TYPE_NORMAL,
		parent_group_id,
		user_rank,
		connected_group,
		is_connected_group,
	)
	.await?;

	let out = GroupCreateOutput {
		group_id,
	};

	echo(out)
}

pub async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupDelete)?;

	let group_data = get_group_user_data_from_req(&req)?;

	group_service::delete_group(
		group_data.group_data.app_id.to_string(),
		group_data.group_data.id.to_string(),
		group_data.user_data.rank,
	)
	.await?;

	echo_success()
}

pub async fn get_user_group_light_data(req: Request) -> JRes<GroupLightServerData>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDataGet)?;

	let group_data = get_group_user_data_from_req(&req)?;

	//no keys fetch here
	echo(group_service::get_user_group_light_data(group_data))
}

pub async fn get_user_group_data(req: Request) -> JRes<GroupServerData>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDataGet)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let out = group_service::get_user_group_data(group_data).await?;

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
pub async fn get_key_update_for_user(req: Request) -> JRes<GroupDataCheckUpdateServerOutput>
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

pub async fn get_user_group_keys(req: Request) -> JRes<Vec<GroupUserKeys>>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserKeys)?;

	//don't get the group data from mw cache, this is done by the model fetch
	let group_data = get_group_user_data_from_req(&req)?;

	let params = get_params(&req)?;
	let last_k_id = get_name_param_from_params(params, "last_k_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let user_keys = group_service::get_user_group_keys(
		group_data.group_data.app_id.to_string(),
		group_data.group_data.id.to_string(),
		group_data.user_data.user_id.to_string(),
		last_fetched_time,
		last_k_id.to_string(),
	)
	.await?;

	echo(user_keys)
}

pub async fn get_user_group_key(req: Request) -> JRes<GroupUserKeys>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserKeys)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let key_id = get_name_param_from_req(&req, "key_id")?;

	let key = group_service::get_user_group_key(
		group_data.group_data.app_id.to_string(),
		group_data.group_data.id.to_string(),
		group_data.user_data.user_id.to_string(),
		key_id.to_string(),
	)
	.await?;

	echo(key)
}

//__________________________________________________________________________________________________

pub async fn stop_invite(req: Request) -> JRes<ServerSuccessOutput>
{
	let group_data = get_group_user_data_from_req(&req)?;

	check_endpoint_with_req(&req, Endpoint::GroupInviteStop)?;

	group_model::stop_invite(
		group_data.group_data.app_id.to_string(),
		group_data.group_data.id.to_string(),
		group_data.user_data.rank,
	)
	.await?;

	let key_group = get_group_cache_key(
		group_data.group_data.app_id.as_str(),
		group_data.group_data.id.as_str(),
	);
	cache::delete(key_group.as_str()).await;

	echo_success()
}

pub async fn get_public_key_data(req: Request) -> JRes<UserPublicKeyDataEntity>
{
	//called from outside of the group mw

	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPublicData)?;

	let group_id = get_name_param_from_req(&req, "group_id")?;

	let data = group_model::get_public_key_data(app_data.app_data.app_id.clone(), group_id.to_string()).await?;

	echo(data)
}

pub async fn get_all_first_level_children(req: Request) -> JRes<Vec<GroupChildrenList>>
{
	check_endpoint_with_req(&req, Endpoint::GroupList)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let params = get_params(&req)?;
	let last_id = get_name_param_from_params(params, "last_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let list = group_service::get_first_level_children(
		group_data.group_data.app_id.clone(),
		group_data.group_data.id.clone(),
		last_fetched_time,
		last_id.to_string(),
	)
	.await?;

	echo(list)
}

//__________________________________________________________________________________________________

//fn which are not related to a specific group

async fn get_all_groups_for(req: Request, user_type: NewUserType) -> JRes<Vec<ListGroups>>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::GroupList)?;

	let params = get_params(&req)?;
	let last_group_id = get_name_param_from_params(params, "last_group_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let user_id = match user_type {
		NewUserType::Normal => {
			let user = get_jwt_data_from_param(&req)?;
			user.id.clone()
		},
		NewUserType::Group => {
			let group_data = get_group_user_data_from_req(&req)?;

			group_data.group_data.id.clone()
		},
	};

	let list = group_model::get_all_groups_to_user(
		app.app_data.app_id.to_string(),
		user_id,
		last_fetched_time,
		last_group_id.to_string(),
	)
	.await?;

	echo(list)
}

pub fn get_all_groups_for_user(req: Request) -> impl Future<Output = JRes<Vec<ListGroups>>>
{
	//this is called from the user without a group id
	get_all_groups_for(req, NewUserType::Normal)
}

pub fn get_all_groups_for_group(req: Request) -> impl Future<Output = JRes<Vec<ListGroups>>>
{
	//get all connected groups (where the group is member)
	get_all_groups_for(req, NewUserType::Group)
}

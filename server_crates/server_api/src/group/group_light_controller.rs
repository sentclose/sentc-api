use std::future::Future;

use rustgram::Request;
use rustgram_server_util::cache;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, JRes, ServerSuccessOutput};
use rustgram_server_util::url_helper::get_name_param_from_req;
use sentc_crypto_common::group::{GroupCreateOutput, GroupDataCheckUpdateServerOutputLight, GroupLightServerData, GroupNewMemberLightInput};
use sentc_crypto_common::GroupId;
use server_api_common::customer_app::{check_endpoint_with_app_options, check_endpoint_with_req, get_app_data_from_req, Endpoint};
use server_api_common::group::{get_group_user_data_from_req, GROUP_TYPE_NORMAL};
use server_api_common::user::get_jwt_data_from_param;
use server_api_common::util::get_group_user_cache_key;

use crate::group::group_user::group_user_model;
use crate::group::{check_invited_group, group_service, group_user_service};
use crate::sentc_group_user_service::NewUserType;
use crate::util::api_res::ApiErrorCodes;

pub fn create_light(req: Request) -> impl Future<Output = JRes<GroupCreateOutput>>
{
	create_group_light(req, None, None, None, false)
}

pub async fn create_child_group_light(req: Request) -> JRes<GroupCreateOutput>
{
	//this is called in the group mw from the parent group id
	let group_data = get_group_user_data_from_req(&req)?;
	let parent_group_id = Some(group_data.group_data.id.to_string());
	let user_rank = Some(group_data.user_data.rank);

	//a connected group can also get children but these children will be a connected group too
	let is_connected_group = group_data.group_data.is_connected_group;

	create_group_light(req, parent_group_id, user_rank, None, is_connected_group).await
}

pub async fn create_connected_group_from_group_light(req: Request) -> JRes<GroupCreateOutput>
{
	/*
	- A connected group is a group where other groups can join or can get invited, not only users.
	- A connected group can also get children (which are marked as connected group too)
	- A connected group cannot be created from an already connected group.
		Because the users of the one connected group cannot access the connected group.
		So only non connected groups can create connected groups.

	- Users can join both groups
	 */

	//the same as parent group, but this time with the group as member, not as parent
	let group_data = get_group_user_data_from_req(&req)?;
	let connected_group_id = Some(group_data.group_data.id.to_string());
	let user_rank = Some(group_data.user_data.rank);

	if group_data.group_data.is_connected_group {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::GroupConnectedFromConnected,
			"Can't create a connected group from a connected group",
		));
	}

	create_group_light(req, None, user_rank, connected_group_id, true).await
}

async fn create_group_light(
	req: Request,
	parent_group_id: Option<GroupId>,
	user_rank: Option<i32>,
	connected_group: Option<GroupId>,
	is_connected_group: bool,
) -> JRes<GroupCreateOutput>
{
	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::GroupCreate)?;

	let user = get_jwt_data_from_param(&req)?;

	let group_id = group_service::create_group_light(
		&app.app_data.app_id,
		&user.id,
		GROUP_TYPE_NORMAL,
		parent_group_id,
		user_rank,
		connected_group,
		is_connected_group,
	)
	.await?;

	echo(GroupCreateOutput {
		group_id,
	})
}

//__________________________________________________________________________________________________

pub fn create_light_force(req: Request) -> impl Future<Output = JRes<GroupCreateOutput>>
{
	create_group_force_light(req, None, None, None, false)
}

pub async fn create_child_group_light_force(req: Request) -> JRes<GroupCreateOutput>
{
	//this is called in the group mw from the parent group id
	let group_data = get_group_user_data_from_req(&req)?;
	let parent_group_id = Some(group_data.group_data.id.to_string());
	let user_rank = Some(group_data.user_data.rank);

	//a connected group can also get children but these children will be a connected group too
	let is_connected_group = group_data.group_data.is_connected_group;

	create_group_force_light(req, parent_group_id, user_rank, None, is_connected_group).await
}

pub async fn create_connected_group_from_group_light_force(req: Request) -> JRes<GroupCreateOutput>
{
	//the same as parent group, but this time with the group as member, not as parent
	let group_data = get_group_user_data_from_req(&req)?;
	let connected_group_id = Some(group_data.group_data.id.to_string());
	let user_rank = Some(group_data.user_data.rank);

	if group_data.group_data.is_connected_group {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::GroupConnectedFromConnected,
			"Can't create a connected group from a connected group",
		));
	}

	create_group_force_light(req, None, user_rank, connected_group_id, true).await
}

async fn create_group_force_light(
	req: Request,
	parent_group_id: Option<GroupId>,
	user_rank: Option<i32>,
	connected_group: Option<GroupId>,
	is_connected_group: bool,
) -> JRes<GroupCreateOutput>
{
	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::ForceServer)?;

	//user id not from jwt but from url param
	let user_id = get_name_param_from_req(&req, "user_id")?;

	let group_id = group_service::create_group_light(
		&app.app_data.app_id,
		user_id,
		GROUP_TYPE_NORMAL,
		parent_group_id,
		user_rank,
		connected_group,
		is_connected_group,
	)
	.await?;

	echo(GroupCreateOutput {
		group_id,
	})
}

//__________________________________________________________________________________________________
//user light
//no re invite here because this is only used when the keys are broken and light got no keys
//no normal join req. no keys are involved for the req, just for to accept

pub fn invite_auto_light(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	auto_invite(req, NewUserType::Normal)
}

pub fn invite_auto_group_light(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	auto_invite(req, NewUserType::Group)
}

async fn auto_invite(mut req: Request, user_type: NewUserType) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupAutoInvite)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let (to_invite, _msg) = check_invited_group(&req, group_data, &user_type).await?;

	let input: GroupNewMemberLightInput = bytes_to_json(&body)?;

	group_user_service::invite_auto_light(group_data, input, to_invite, user_type).await?;

	echo_success()
}

pub async fn invite_auto_group_force_light(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::ForceServer)?;

	let group_data = get_group_user_data_from_req(&req)?;
	let to_invite = get_name_param_from_req(&req, "invited_group")?;

	let input: GroupNewMemberLightInput = bytes_to_json(&body)?;

	group_user_service::invite_auto_light(group_data, input, to_invite, NewUserType::Group).await?;

	echo_success()
}

pub fn invite_group_to_group_from_server_light(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	invite_to_group_from_server(req, NewUserType::Group)
}

pub fn invite_user_to_group_from_server_light(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	invite_to_group_from_server(req, NewUserType::Normal)
}

async fn invite_to_group_from_server(mut req: Request, user_type: NewUserType) -> JRes<ServerSuccessOutput>
{
	//invite a user from server without being a member in this group

	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::ForceServer)?;

	let to_invite = get_name_param_from_req(&req, "to_invite")?;
	let group_data = get_group_user_data_from_req(&req)?;

	let input: GroupNewMemberLightInput = bytes_to_json(&body)?;

	group_user_service::invite_auto_light(group_data, input, to_invite, user_type).await?;

	echo_success()
}

pub fn invite_request_light(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	invite(req, NewUserType::Normal)
}

pub fn invite_request_to_group_light(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	invite(req, NewUserType::Group)
}

async fn invite(mut req: Request, user_type: NewUserType) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	check_endpoint_with_req(&req, Endpoint::GroupInvite)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let (to_invite, _msg) = check_invited_group(&req, group_data, &user_type).await?;

	let input: GroupNewMemberLightInput = bytes_to_json(&body)?;

	group_user_service::invite_request_light(group_data, input, to_invite, user_type).await?;

	echo_success()
}

pub async fn accept_join_req_light(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupAcceptJoinReq)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let join_user = get_name_param_from_req(&req, "join_user")?;

	let input: GroupNewMemberLightInput = bytes_to_json(&body)?;

	let rank = input.rank.unwrap_or(4);

	if rank < 1 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserRank,
			"User group rank got the wrong format",
		));
	}

	if rank < group_data.user_data.rank {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserRank,
			"The set rank cannot be higher than your rank",
		));
	}

	group_user_model::accept_join_req_light(&group_data.group_data.id, join_user, rank, group_data.user_data.rank).await?;

	//delete user group cache. no need to delete the user group cache again for upload session,
	// because after this fn the user is already registered
	let key_user = get_group_user_cache_key(&group_data.group_data.app_id, &group_data.group_data.id, join_user);

	cache::delete(&key_user).await?;

	echo_success()
}

//__________________________________________________________________________________________________

pub async fn get_user_group_light_data(req: Request) -> JRes<GroupLightServerData>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDataGet)?;

	let group_data = get_group_user_data_from_req(&req)?;

	//no keys fetch here
	echo(group_service::get_user_group_light_data(group_data))
}

pub async fn get_update_for_user_light(req: Request) -> JRes<GroupDataCheckUpdateServerOutputLight>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserUpdateCheck)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let out = GroupDataCheckUpdateServerOutputLight {
		rank: group_data.user_data.rank,
	};

	echo(out)
}

use std::future::Future;

use rustgram::Request;
use rustgram_server_util::cache;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, AppRes, JRes, ServerSuccessOutput};
use rustgram_server_util::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params, get_time_from_url_param};
use sentc_crypto_common::group::{
	GroupAcceptJoinReqServerOutput,
	GroupChangeRankServerInput,
	GroupInviteServerOutput,
	GroupKeysForNewMember,
	GroupKeysForNewMemberServerInput,
};
use server_api_common::customer_app::{check_endpoint_with_app_options, check_endpoint_with_req, get_app_data_from_req, Endpoint};
use server_api_common::group::get_group_user_data_from_req;
use server_api_common::group::group_entities::InternalGroupDataComplete;
use server_api_common::user::get_jwt_data_from_param;
use server_api_common::util::get_group_user_cache_key;

use crate::group::group_entities::{GroupInviteReq, GroupJoinReq, GroupUserListItem};
use crate::group::group_model;
use crate::group::group_user::{group_user_model, group_user_service};
use crate::group::group_user_service::{InsertNewUserType, NewUserType};
use crate::util::api_res::ApiErrorCodes;

pub async fn get_group_member(req: Request) -> JRes<Vec<GroupUserListItem>>
{
	let group_data = get_group_user_data_from_req(&req)?;

	let params = get_params(&req)?;
	let last_user_id = get_name_param_from_params(params, "last_user_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let list_fetch = group_user_service::get_group_member(
		&group_data.group_data.id,
		&group_data.user_data.user_id,
		last_fetched_time,
		last_user_id,
	)
	.await?;

	echo(list_fetch)
}

pub async fn get_single_group_member(req: Request) -> JRes<GroupUserListItem>
{
	//force direct request from server
	check_endpoint_with_req(&req, Endpoint::ForceServer)?;

	let app_data = get_app_data_from_req(&req)?;

	let group_id = get_name_param_from_req(&req, "group_id")?;

	let user_id = get_name_param_from_req(&req, "user_id")?;

	let out = group_user_service::get_single_group_member(&app_data.app_data.app_id, group_id, user_id).await?;

	echo(out)
}

//__________________________________________________________________________________________________

pub fn invite_auto(req: Request) -> impl Future<Output = JRes<GroupInviteServerOutput>>
{
	auto_invite(req, NewUserType::Normal)
}

pub fn invite_auto_group(req: Request) -> impl Future<Output = JRes<GroupInviteServerOutput>>
{
	auto_invite(req, NewUserType::Group)
}

async fn auto_invite(mut req: Request, user_type: NewUserType) -> JRes<GroupInviteServerOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupAutoInvite)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let (to_invite, msg) = check_invited_group(&req, group_data, &user_type).await?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	let session_id = group_user_service::invite_auto(group_data, input, to_invite, user_type, false).await?;

	let out = GroupInviteServerOutput {
		session_id,
		message: msg.to_string() + " was auto invited. The user is now a group member.",
	};

	echo(out)
}

pub async fn invite_auto_group_force(mut req: Request) -> JRes<GroupInviteServerOutput>
{
	//the same as the other but without the restriction that a group must be a connected group
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::ForceServer)?;

	let group_data = get_group_user_data_from_req(&req)?;
	let to_invite = get_name_param_from_req(&req, "invited_group")?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	let session_id = group_user_service::invite_auto(group_data, input, to_invite, NewUserType::Group, false).await?;

	let out = GroupInviteServerOutput {
		session_id,
		message: "Group was invited. Please wait until the user accepts the invite.".to_string(),
	};

	echo(out)
}

pub fn invite_group_to_group_from_server(req: Request) -> impl Future<Output = JRes<GroupInviteServerOutput>>
{
	invite_to_group_from_server(req, NewUserType::Group)
}

pub fn invite_user_to_group_from_server(req: Request) -> impl Future<Output = JRes<GroupInviteServerOutput>>
{
	invite_to_group_from_server(req, NewUserType::Normal)
}

async fn invite_to_group_from_server(mut req: Request, user_type: NewUserType) -> JRes<GroupInviteServerOutput>
{
	//just invite a user without jwt check and without group restriction check
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::ForceServer)?;
	let group_data = get_group_user_data_from_req(&req)?;
	let to_invite = get_name_param_from_req(&req, "to_invite")?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	let session_id = group_user_service::invite_auto(group_data, input, to_invite, user_type, false).await?;

	let out = GroupInviteServerOutput {
		session_id,
		message: "Was invited. Please wait until the user accepts the invite.".to_string(),
	};

	echo(out)
}

pub(crate) async fn check_invited_group<'a>(
	req: &'a Request,
	group_data: &InternalGroupDataComplete,
	user_type: &NewUserType,
) -> AppRes<(&'a str, &'a str)>
{
	match *user_type {
		NewUserType::Normal => Ok((get_name_param_from_req(req, "invited_user")?, "Group")),
		NewUserType::Group => {
			//only connected groups can have other groups as member
			//check in the model if the group to invite a non-connected group
			if !group_data.group_data.is_connected_group {
				return Err(ServerCoreError::new_msg(
					400,
					ApiErrorCodes::GroupJoinAsConnectedGroup,
					"Can't invite another group when this group is not a connected group",
				));
			}

			let to_invite = get_name_param_from_req(req, "invited_group")?;

			//get the int user type, and if it is a group, check if the group is a non-connected group
			// do it with the model because we don't get any info about the group until now
			let cg = group_user_service::check_is_connected_group(to_invite).await?;

			if cg == 1 {
				return Err(ServerCoreError::new_msg(
					400,
					ApiErrorCodes::GroupJoinAsConnectedGroup,
					"Can't invite group when the group is a connected group",
				));
			}

			Ok((to_invite, "User"))
		},
	}
}

pub fn invite_request(req: Request) -> impl Future<Output = JRes<GroupInviteServerOutput>>
{
	//no, the acceptance invite, but the keys are prepared for the invited user
	//don't save this value in the group user keys table, but in the invite table

	invite(req, NewUserType::Normal)
}

pub fn invite_request_to_group(req: Request) -> impl Future<Output = JRes<GroupInviteServerOutput>>
{
	invite(req, NewUserType::Group)
}

async fn invite(mut req: Request, user_type: NewUserType) -> JRes<GroupInviteServerOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupInvite)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let (to_invite, msg) = check_invited_group(&req, group_data, &user_type).await?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	let session_id = group_user_service::invite_request(group_data, input, to_invite, user_type).await?;

	let out = GroupInviteServerOutput {
		session_id,
		message: msg.to_string() + " was invited. Please wait until the user accepts the invite.",
	};

	echo(out)
}

pub fn get_invite_req(req: Request) -> impl Future<Output = JRes<Vec<GroupInviteReq>>>
{
	get_invite_req_pri(req, NewUserType::Normal)
}

pub fn get_invite_req_for_group(req: Request) -> impl Future<Output = JRes<Vec<GroupInviteReq>>>
{
	//call this from the group which gets all the invite req

	get_invite_req_pri(req, NewUserType::Group)
}

async fn get_invite_req_pri(req: Request, user_type: NewUserType) -> JRes<Vec<GroupInviteReq>>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::GroupInvite)?;

	let id_to_check = match user_type {
		NewUserType::Normal => {
			let user = get_jwt_data_from_param(&req)?;

			&user.id
		},
		NewUserType::Group => {
			let group_data = get_group_user_data_from_req(&req)?;

			group_model::check_group_rank(group_data.user_data.rank, 1)?;

			&group_data.group_data.id
		},
	};

	let params = get_params(&req)?;
	let last_group_id = get_name_param_from_params(params, "last_group_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let out = group_user_service::get_invite_req(&app.app_data.app_id, id_to_check, last_fetched_time, last_group_id).await?;

	echo(out)
}

pub fn reject_invite(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	reject_invite_pri(req, NewUserType::Normal)
}

pub fn reject_invite_group(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	reject_invite_pri(req, NewUserType::Group)
}

async fn reject_invite_pri(req: Request, user_type: NewUserType) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupRejectInvite)?;

	let (key_id_to_reject, user_id) = match user_type {
		NewUserType::Normal => {
			let user = get_jwt_data_from_param(&req)?;
			("group_id", &user.id)
		},
		NewUserType::Group => {
			let group_data = get_group_user_data_from_req(&req)?;

			group_model::check_group_rank(group_data.user_data.rank, 1)?;

			("group_id_to_reject", &group_data.group_data.id)
		},
	};

	let group_id = get_name_param_from_req(&req, key_id_to_reject)?;

	group_user_model::reject_invite(group_id, user_id).await?;

	echo_success()
}

pub fn accept_invite(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	accept_invite_pri(req, NewUserType::Normal)
}

pub fn accept_invite_group(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	accept_invite_pri(req, NewUserType::Group)
}

async fn accept_invite_pri(req: Request, user_type: NewUserType) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::GroupAcceptInvite)?;

	let (key_id_to_accept, user_id) = match user_type {
		NewUserType::Normal => {
			let user = get_jwt_data_from_param(&req)?;

			("group_id", &user.id)
		},
		NewUserType::Group => {
			let group_data = get_group_user_data_from_req(&req)?;

			group_model::check_group_rank(group_data.user_data.rank, 1)?;

			("group_id_to_join", &group_data.group_data.id)
		},
	};

	let group_id = get_name_param_from_req(&req, key_id_to_accept)?;
	group_user_service::accept_invite(&app.app_data.app_id, group_id, user_id).await?;

	echo_success()
}

pub fn re_invite_auto(req: Request) -> impl Future<Output = JRes<GroupInviteServerOutput>>
{
	re_invite_user(req, NewUserType::Normal)
}

pub fn re_invite_auto_group(req: Request) -> impl Future<Output = JRes<GroupInviteServerOutput>>
{
	re_invite_user(req, NewUserType::Group)
}

async fn re_invite_user(mut req: Request, user_type: NewUserType) -> JRes<GroupInviteServerOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupAutoInvite)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let (to_invite, msg) = check_invited_group(&req, group_data, &user_type).await?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	let session_id = group_user_service::re_invite_user(group_data, input, to_invite, user_type).await?;

	let out = GroupInviteServerOutput {
		session_id,
		message: msg.to_string() + " was re invited. The user is a group member again.",
	};

	echo(out)
}

pub async fn re_invite_force(mut req: Request) -> JRes<GroupInviteServerOutput>
{
	//the same as the other but without the restriction that a group must be a connected group
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::ForceServer)?;

	let group_data = get_group_user_data_from_req(&req)?;
	let to_invite = get_name_param_from_req(&req, "invited_group")?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	let session_id = group_user_service::re_invite_user(group_data, input, to_invite, NewUserType::Group).await?;

	let out = GroupInviteServerOutput {
		session_id,
		message: "Group was re invited. The user is a group member again.".to_string(),
	};

	echo(out)
}

//__________________________________________________________________________________________________

pub fn join_req(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	join_req_pri(req, NewUserType::Normal)
}

pub fn join_req_as_group(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	//doing join req from a group to another group to join it as a member
	join_req_pri(req, NewUserType::Group)
}

async fn join_req_pri(req: Request, user_type: NewUserType) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::GroupJoinReq)?;

	let (key_for_group_id_to_join, id) = match user_type {
		NewUserType::Group => {
			let group_data = get_group_user_data_from_req(&req)?;
			//only high member can send join req
			group_model::check_group_rank(group_data.user_data.rank, 1)?;

			//check here if this group is a connected group
			//only normal groups can be a member in connected groups but not connected groups in another group
			//check in the model if the group to join is a connected group
			if group_data.group_data.is_connected_group {
				return Err(ServerCoreError::new_msg(
					400,
					ApiErrorCodes::GroupJoinAsConnectedGroup,
					"Can't join another group when this group is a connected group",
				));
			}

			("group_id_to_join", &group_data.group_data.id)
		},
		NewUserType::Normal => {
			let user = get_jwt_data_from_param(&req)?;

			("group_id", &user.id)
		},
	};

	let group_id_to_join = get_name_param_from_req(&req, key_for_group_id_to_join)?;

	group_user_model::join_req(&app.app_data.app_id, group_id_to_join, id, user_type).await?;

	echo_success()
}

pub async fn get_join_req(req: Request) -> JRes<Vec<GroupJoinReq>>
{
	check_endpoint_with_req(&req, Endpoint::GroupJoinReq)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let params = get_params(&req)?;
	let last_user_id = get_name_param_from_params(params, "last_user_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let reqs = group_user_model::get_join_req(
		&group_data.group_data.id,
		last_fetched_time,
		last_user_id,
		group_data.user_data.rank,
	)
	.await?;

	echo(reqs)
}

pub fn get_sent_join_req_for_user(req: Request) -> impl Future<Output = JRes<Vec<GroupInviteReq>>>
{
	//list all join req of the user
	get_sent_join_req(req, NewUserType::Normal)
}

pub fn get_sent_join_req_for_group(req: Request) -> impl Future<Output = JRes<Vec<GroupInviteReq>>>
{
	//list all join req of a group as member
	get_sent_join_req(req, NewUserType::Group)
}

async fn get_sent_join_req(req: Request, user_type: NewUserType) -> JRes<Vec<GroupInviteReq>>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::GroupJoinReq)?;

	let id_to_check = check_join_req_access(&req, user_type)?;

	let params = get_params(&req)?;
	let last_group_id = get_name_param_from_params(params, "last_group_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let out = group_user_model::get_sent_join_req(&app.app_data.app_id, id_to_check, last_fetched_time, last_group_id).await?;

	echo(out)
}

pub async fn reject_join_req(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupRejectJoinReq)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let join_user = get_name_param_from_req(&req, "join_user")?;

	group_user_model::reject_join_req(&group_data.group_data.id, join_user, group_data.user_data.rank).await?;

	echo_success()
}

pub async fn accept_join_req(mut req: Request) -> JRes<GroupAcceptJoinReqServerOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupAcceptJoinReq)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let join_user = get_name_param_from_req(&req, "join_user")?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	if input.keys.is_empty() {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::GroupNoKeys,
			"No group keys for the user",
		));
	}

	if input.keys.len() > 100 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::GroupTooManyKeys,
			"Too many group keys for the user. Split the keys and use pagination",
		));
	}

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

	let session_id = group_user_model::accept_join_req(
		&group_data.group_data.id,
		join_user,
		input.keys,
		input.key_session,
		rank,
		group_data.user_data.rank,
	)
	.await?;

	let out = GroupAcceptJoinReqServerOutput {
		session_id,
		message: "The join request was accepted. The user is now a member of this group.".to_string(),
	};

	//delete user group cache. no need to delete the user group cache again for the upload session,
	// because after this fn the user is already registered
	let key_user = get_group_user_cache_key(&group_data.group_data.app_id, &group_data.group_data.id, join_user);

	cache::delete(&key_user).await?;

	echo(out)
}

pub fn delete_sent_join_req_for_user(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	delete_sent_join_req(req, NewUserType::Normal)
}

pub fn delete_sent_join_req_for_group(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	delete_sent_join_req(req, NewUserType::Group)
}

async fn delete_sent_join_req(req: Request, user_type: NewUserType) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::GroupJoinReq)?;

	let id_to_check = check_join_req_access(&req, user_type)?;

	let id = get_name_param_from_req(&req, "join_req_id")?;

	group_user_model::delete_sent_join_req(&app.app_data.app_id, id_to_check, id).await?;

	echo_success()
}

fn check_join_req_access(req: &Request, user_type: NewUserType) -> AppRes<&String>
{
	match user_type {
		NewUserType::Normal => {
			let user = get_jwt_data_from_param(req)?;

			Ok(&user.id)
		},
		NewUserType::Group => {
			let group_data = get_group_user_data_from_req(req)?;

			group_model::check_group_rank(group_data.user_data.rank, 1)?;

			Ok(&group_data.group_data.id)
		},
	}
}

//__________________________________________________________________________________________________

pub fn insert_user_keys_via_session_invite(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	insert_user_keys_via_session(req, InsertNewUserType::Invite)
}

pub fn insert_user_keys_via_session_join_req(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	insert_user_keys_via_session(req, InsertNewUserType::Join)
}

//__________________________________________________________________________________________________

pub async fn leave_group(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupLeave)?;

	let group_data = get_group_user_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	group_user_service::leave_group(group_data, Some(&user.id)).await?;

	echo_success()
}

pub fn kick_user_from_group(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	kick_user(req, Endpoint::GroupUserDelete, "user_id")
}

pub fn kick_user_from_group_forced(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	kick_user(req, Endpoint::ForceServer, "user_to_kick")
}

async fn kick_user(req: Request, endpoint: Endpoint, req_param: &str) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, endpoint)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let user_id = get_name_param_from_req(&req, req_param)?;

	group_user_service::kick_user_from_group(group_data, user_id, false).await?;

	echo_success()
}

//__________________________________________________________________________________________________

pub async fn change_rank(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupChangeRank)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let input: GroupChangeRankServerInput = bytes_to_json(&body)?;

	group_user_service::change_rank(group_data, input.changed_user_id, input.new_rank).await?;

	echo_success()
}

//__________________________________________________________________________________________________

async fn insert_user_keys_via_session(mut req: Request, insert_type: InsertNewUserType) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	let group_data = get_group_user_data_from_req(&req)?;

	let key_session_id = get_name_param_from_req(&req, "key_session_id")?;

	let input: Vec<GroupKeysForNewMember> = bytes_to_json(&body)?;

	group_user_service::insert_user_keys_via_session(&group_data.group_data.id, key_session_id, insert_type, input).await?;

	echo_success()
}

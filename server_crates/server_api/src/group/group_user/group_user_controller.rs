use std::future::Future;

use rustgram::Request;
use sentc_crypto_common::group::{
	GroupAcceptJoinReqServerOutput,
	GroupChangeRankServerInput,
	GroupInviteServerOutput,
	GroupKeysForNewMember,
	GroupKeysForNewMemberServerInput,
};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use server_core::cache;
use server_core::input_helper::{bytes_to_json, get_raw_body};
use server_core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};

use crate::customer_app::app_util::{check_endpoint_with_app_options, check_endpoint_with_req, get_app_data_from_req, Endpoint};
use crate::group::group_entities::{GroupInviteReq, GroupJoinReq, GroupUserListItem};
use crate::group::group_user::{group_user_model, group_user_service};
use crate::group::group_user_service::{InsertNewUserType, NewUserType};
use crate::group::{get_group_user_data_from_req, group_model};
use crate::user::jwt::get_jwt_data_from_param;
use crate::util::api_res::{echo, echo_success, ApiErrorCodes, AppRes, HttpErr, JRes};
use crate::util::get_group_user_cache_key;

pub async fn get_group_member(req: Request) -> JRes<Vec<GroupUserListItem>>
{
	let group_data = get_group_user_data_from_req(&req)?;

	let params = get_params(&req)?;
	let last_user_id = get_name_param_from_params(params, "last_user_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let list_fetch = group_user_model::get_group_member(
		&group_data.group_data.id,
		&group_data.user_data.user_id,
		last_fetched_time,
		last_user_id,
	)
	.await?;

	echo(list_fetch)
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

	let (key, msg) = match user_type {
		NewUserType::Normal => ("invited_user", "Group"),
		NewUserType::Group => ("invited_group", "User"),
	};

	let to_invite = get_name_param_from_req(&req, key)?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	let session_id = group_user_service::invite_auto(group_data, input, to_invite, user_type).await?;

	let out = GroupInviteServerOutput {
		session_id,
		message: msg.to_string() + " was invited. Please wait until the user accepts the invite.",
	};

	echo(out)
}

pub fn invite_request(req: Request) -> impl Future<Output = JRes<GroupInviteServerOutput>>
{
	//no the accept invite, but the keys are prepared for the invited user
	//don't save this values in the group user keys table, but in the invite table

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

	let (key, msg) = match user_type {
		NewUserType::Normal => ("invited_user", "Group"),
		NewUserType::Group => ("invited_group", "User"),
	};

	let to_invite = get_name_param_from_req(&req, key)?;

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
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

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
	group_user_model::accept_invite(group_id, user_id).await?;

	//delete the cache here so the user can join the group
	let key_user = get_group_user_cache_key(&app.app_data.app_id, group_id, user_id);

	cache::delete(&key_user).await;

	echo_success()
}

//__________________________________________________________________________________________________

pub fn join_req(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	join_req_pri(req, NewUserType::Normal)
}

pub fn join_req_as_group(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	//doing join req from a group to another group to join it as member
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

			//check here if this group ia a connected group
			//only normal groups can be a member in connected groups but not connected groups in another group
			//check in the model if the group to join is a connected group
			if group_data.group_data.is_connected_group {
				return Err(HttpErr::new(
					400,
					ApiErrorCodes::GroupJoinAsConnectedGroup,
					"Can't join another group when this group is a connected group".to_string(),
					None,
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
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

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
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

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
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupNoKeys,
			"No group keys for the user".to_string(),
			None,
		));
	}

	if input.keys.len() > 100 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupTooManyKeys,
			"Too many group keys for the user. Split the keys and use pagination".to_string(),
			None,
		));
	}

	let session_id = group_user_model::accept_join_req(
		&group_data.group_data.id,
		join_user,
		input.keys,
		input.key_session,
		group_data.user_data.rank,
	)
	.await?;

	let out = GroupAcceptJoinReqServerOutput {
		session_id,
		message: "The join request was accepted. The user is now a member of this group.".to_string(),
	};

	//delete user group cache. no need to delete the user group cache again for upload session,
	// because after this fn the user is already registered
	let key_user = get_group_user_cache_key(&group_data.group_data.app_id, &group_data.group_data.id, join_user);

	cache::delete(&key_user).await;

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

pub async fn kick_user_from_group(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDelete)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;

	group_user_model::kick_user_from_group(&group_data.group_data.id, user_id, group_data.user_data.rank).await?;

	//delete the user cache
	let key_group = get_group_user_cache_key(&group_data.group_data.app_id, &group_data.group_data.id, user_id);

	cache::delete(&key_group).await;

	echo_success()
}

//__________________________________________________________________________________________________

/**
Update the user rank. The rank of a creator cannot changed.

When deleting the cache for this group, and the group got children then for all children the rank must be updated too.
This is done because we use a reference to the parent group when we look for the user rank in the group mw.
If this user is not in a parent group -> this wouldn't effect any groups
 */
pub async fn change_rank(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupChangeRank)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let input: GroupChangeRankServerInput = bytes_to_json(&body)?;

	group_user_model::update_rank(
		&group_data.group_data.id,
		group_data.user_data.rank,
		&input.changed_user_id,
		input.new_rank,
	)
	.await?;

	//delete user cache of the changed user
	let key_group = get_group_user_cache_key(
		&group_data.group_data.app_id,
		&group_data.group_data.id,
		&input.changed_user_id,
	);

	cache::delete(&key_group).await;

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

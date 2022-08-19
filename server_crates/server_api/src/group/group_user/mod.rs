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

use crate::customer_app::app_util::{check_endpoint_with_req, Endpoint};
use crate::group::get_group_user_data_from_req;
use crate::group::group_entities::{GroupInviteReq, GroupJoinReq, GroupUserListItem};
use crate::group::group_user::group_user_model::InsertNewUserType;
use crate::user::jwt::get_jwt_data_from_param;
use crate::util::api_res::{echo, echo_success, ApiErrorCodes, HttpErr, JRes};
use crate::util::get_group_user_cache_key;

mod group_user_model;
pub mod group_user_service;

pub(crate) async fn get_group_member(req: Request) -> JRes<Vec<GroupUserListItem>>
{
	let group_data = get_group_user_data_from_req(&req)?;

	let params = get_params(&req)?;
	let last_user_id = get_name_param_from_params(&params, "last_user_id")?;
	let last_fetched_time = get_name_param_from_params(&params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let list_fetch = group_user_model::get_group_member(
		group_data.group_data.id.to_string(),
		group_data.user_data.user_id.to_string(),
		last_fetched_time,
		last_user_id.to_string(),
	)
	.await?;

	echo(list_fetch)
}

//__________________________________________________________________________________________________

pub(crate) async fn invite_auto(mut req: Request) -> JRes<GroupInviteServerOutput>
{
	let body = get_raw_body(&mut req).await?;

	//TODO new endpoint
	check_endpoint_with_req(&req, Endpoint::GroupInvite)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let invited_user = get_name_param_from_req(&req, "invited_user")?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	let session_id = group_user_service::invite_auto(group_data, input, invited_user.to_string()).await?;

	let out = GroupInviteServerOutput {
		session_id,
		message: "User was invited. Please wait until the user accepts the invite.".to_string(),
	};

	echo(out)
}

pub(crate) async fn invite_request(mut req: Request) -> JRes<GroupInviteServerOutput>
{
	//no the accept invite, but the keys are prepared for the invited user
	//don't save this values in the group user keys table, but in the invite table

	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupInvite)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let invited_user = get_name_param_from_req(&req, "invited_user")?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	let session_id = group_user_service::invite_request(group_data, input, invited_user.to_string()).await?;

	let out = GroupInviteServerOutput {
		session_id,
		message: "User was invited. Please wait until the user accepts the invite.".to_string(),
	};

	echo(out)
}

pub(crate) async fn get_invite_req(req: Request) -> JRes<Vec<GroupInviteReq>>
{
	check_endpoint_with_req(&req, Endpoint::GroupInvite)?;

	//called from the invited user not the group admin

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

	let out = group_user_service::get_invite_req(
		user.sub.to_string(),
		user.id.to_string(),
		last_fetched_time,
		last_group_id.to_string(),
	)
	.await?;

	echo(out)
}

pub(crate) async fn reject_invite(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupRejectInvite)?;

	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	group_user_model::reject_invite(group_id.to_string(), user.id.to_string()).await?;

	echo_success()
}

pub(crate) async fn accept_invite(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupAcceptInvite)?;

	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	group_user_model::accept_invite(group_id.to_string(), user.id.to_string()).await?;

	//delete the cache here so the user can join the group
	let key_user = get_group_user_cache_key(user.sub.as_str(), group_id, user.id.as_str());

	cache::delete(&key_user).await;

	echo_success()
}

//__________________________________________________________________________________________________

pub(crate) async fn join_req(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupJoinReq)?;

	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	group_user_model::join_req(group_id.to_string(), user.id.to_string()).await?;

	echo_success()
}

pub(crate) async fn get_join_req(req: Request) -> JRes<Vec<GroupJoinReq>>
{
	check_endpoint_with_req(&req, Endpoint::GroupJoinReq)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let params = get_params(&req)?;
	let last_user_id = get_name_param_from_params(&params, "last_user_id")?;
	let last_fetched_time = get_name_param_from_params(&params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let reqs = group_user_model::get_join_req(
		group_data.group_data.id.to_string(),
		last_fetched_time,
		last_user_id.to_string(),
		group_data.user_data.rank,
	)
	.await?;

	echo(reqs)
}

pub(crate) async fn reject_join_req(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupRejectJoinReq)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let join_user = get_name_param_from_req(&req, "join_user")?;

	group_user_model::reject_join_req(
		group_data.group_data.id.to_string(),
		join_user.to_string(),
		group_data.user_data.rank,
	)
	.await?;

	echo_success()
}
pub(crate) async fn accept_join_req(mut req: Request) -> JRes<GroupAcceptJoinReqServerOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupAcceptJoinReq)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let join_user = get_name_param_from_req(&req, "join_user")?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	if input.keys.len() == 0 {
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
		group_data.group_data.id.to_string(),
		join_user.to_string(),
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
	let key_user = get_group_user_cache_key(
		group_data.group_data.app_id.as_str(),
		group_data.group_data.id.as_str(),
		join_user,
	);

	cache::delete(&key_user).await;

	echo(out)
}

//__________________________________________________________________________________________________

pub(crate) fn insert_user_keys_via_session_invite(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	insert_user_keys_via_session(req, InsertNewUserType::Invite)
}

pub(crate) fn insert_user_keys_via_session_join_req(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	insert_user_keys_via_session(req, InsertNewUserType::Join)
}

//__________________________________________________________________________________________________

pub(crate) async fn leave_group(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupLeave)?;

	let group_data = get_group_user_data_from_req(&req)?;

	group_user_model::user_leave_group(
		group_data.group_data.id.to_string(),
		group_data.user_data.user_id.to_string(),
		group_data.user_data.rank,
	)
	.await?;

	//delete the user cache
	let key_group = get_group_user_cache_key(
		group_data.group_data.app_id.as_str(),
		group_data.group_data.id.as_str(),
		group_data.user_data.user_id.as_str(),
	);

	cache::delete(key_group.as_str()).await;

	echo_success()
}

pub(crate) async fn kick_user_from_group(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDelete)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;

	group_user_model::kick_user_from_group(
		group_data.group_data.id.to_string(),
		user_id.to_string(),
		group_data.user_data.rank,
	)
	.await?;

	//delete the user cache
	let key_group = get_group_user_cache_key(
		group_data.group_data.app_id.as_str(),
		group_data.group_data.id.as_str(),
		user_id,
	);

	cache::delete(key_group.as_str()).await;

	echo_success()
}

//__________________________________________________________________________________________________

/**
Update the user rank. The rank of a creator cannot changed.

When deleting the cache for this group, and the group got children then for all children the rank must be updated too.
This is done because we use a reference to the parent group when we look for the user rank in the group mw.
If this user is not in a parent group -> this wouldn't effect any groups
*/
pub(crate) async fn change_rank(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupChangeRank)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let input: GroupChangeRankServerInput = bytes_to_json(&body)?;

	group_user_model::update_rank(
		group_data.group_data.id.to_string(),
		group_data.user_data.rank,
		input.changed_user_id.to_string(),
		input.new_rank,
	)
	.await?;

	//delete user cache of the changed user
	let key_group = get_group_user_cache_key(
		group_data.group_data.app_id.as_str(),
		group_data.group_data.id.as_str(),
		input.changed_user_id.as_str(),
	);

	cache::delete(key_group.as_str()).await;

	echo_success()
}

//__________________________________________________________________________________________________

async fn insert_user_keys_via_session(mut req: Request, insert_type: InsertNewUserType) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	let group_data = get_group_user_data_from_req(&req)?;

	let key_session_id = get_name_param_from_req(&req, "key_session_id")?;

	let input: Vec<GroupKeysForNewMember> = bytes_to_json(&body)?;

	if input.len() == 0 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupNoKeys,
			"No group keys for the user".to_string(),
			None,
		));
	}

	if input.len() > 100 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupTooManyKeys,
			"Too many group keys for the user. Split the keys and use pagination".to_string(),
			None,
		));
	}

	group_user_model::insert_user_keys_via_session(
		group_data.group_data.id.to_string(),
		key_session_id.to_string(),
		input,
		insert_type,
	)
	.await?;

	echo_success()
}

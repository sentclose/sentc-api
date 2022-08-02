use std::future::Future;

use rustgram::Request;
use sentc_crypto_common::group::{
	GroupAcceptJoinReqServerOutput,
	GroupInviteReqList,
	GroupInviteServerOutput,
	GroupJoinReqList,
	GroupKeysForNewMember,
	GroupKeysForNewMemberServerInput,
};
use sentc_crypto_common::server_default::ServerSuccessOutput;

use crate::core::api_res::{echo, echo_success, ApiErrorCodes, HttpErr, JRes};
use crate::core::cache;
use crate::core::cache::INTERNAL_GROUP_USER_DATA_CACHE;
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};
use crate::group::get_group_user_data_from_req;
use crate::group::group_entities::{GroupNewUserType, GROUP_INVITE_TYPE_INVITE_REQ, GROUP_INVITE_TYPE_JOIN_REQ};
use crate::user::jwt::get_jwt_data_from_param;

mod group_user_model;

pub(crate) async fn invite_request(mut req: Request) -> JRes<GroupInviteServerOutput>
{
	//no the accept invite, but the keys are prepared for the invited user
	//don't save this values in the group user keys table, but in the invite table

	let body = get_raw_body(&mut req).await?;

	let group_data = get_group_user_data_from_req(&req)?;

	let invited_user = get_name_param_from_req(&req, "invited_user")?;

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

	let session_id = group_user_model::invite_request(
		group_data.group_data.id.to_string(),
		invited_user.to_string(),
		input.keys,
		input.key_session,
		group_data.user_data.rank,
	)
	.await?;

	let out = GroupInviteServerOutput {
		session_id,
		message: "User was invited. Please wait until the user accepts the invite.".to_string(),
	};

	echo(out)
}

pub(crate) async fn get_invite_req(req: Request) -> JRes<Vec<GroupInviteReqList>>
{
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

	let reqs = group_user_model::get_invite_req_to_user(
		user.sub.to_string(),
		user.id.to_string(),
		last_fetched_time,
		last_group_id.to_string(),
	)
	.await?;

	let mut out: Vec<GroupInviteReqList> = Vec::with_capacity(reqs.len());
	for item in reqs {
		out.push(item.into());
	}

	echo(out)
}

pub(crate) async fn reject_invite(req: Request) -> JRes<ServerSuccessOutput>
{
	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	group_user_model::reject_invite(group_id.to_string(), user.id.to_string()).await?;

	echo_success()
}

pub(crate) async fn accept_invite(req: Request) -> JRes<ServerSuccessOutput>
{
	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	group_user_model::accept_invite(group_id.to_string(), user.id.to_string()).await?;

	echo_success()
}

//__________________________________________________________________________________________________

pub(crate) async fn join_req(req: Request) -> JRes<ServerSuccessOutput>
{
	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	group_user_model::join_req(group_id.to_string(), user.id.to_string()).await?;

	echo_success()
}

pub(crate) async fn get_join_req(req: Request) -> JRes<Vec<GroupJoinReqList>>
{
	let group_data = get_group_user_data_from_req(&req)?;

	let last_fetched_time = get_name_param_from_req(&req, "last_fetched_time")?;
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
		group_data.user_data.rank,
	)
	.await?;

	let mut req_out: Vec<GroupJoinReqList> = Vec::with_capacity(reqs.len());
	for item in reqs {
		req_out.push(item.into());
	}

	echo(req_out)
}

pub(crate) async fn reject_join_req(req: Request) -> JRes<ServerSuccessOutput>
{
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

	echo(out)
}

//__________________________________________________________________________________________________

pub(crate) fn insert_user_keys_via_session_invite(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	insert_user_keys_via_session(req, GROUP_INVITE_TYPE_INVITE_REQ)
}

pub(crate) fn insert_user_keys_via_session_join_req(req: Request) -> impl Future<Output = JRes<ServerSuccessOutput>>
{
	insert_user_keys_via_session(req, GROUP_INVITE_TYPE_JOIN_REQ)
}

//__________________________________________________________________________________________________

pub(crate) async fn leave_group(req: Request) -> JRes<ServerSuccessOutput>
{
	let group_data = get_group_user_data_from_req(&req)?;

	group_user_model::user_leave_group(
		group_data.group_data.id.to_string(),
		group_data.user_data.user_id.to_string(),
		group_data.user_data.rank,
	)
	.await?;

	//delete the user cache
	let key_group = INTERNAL_GROUP_USER_DATA_CACHE.to_string() +
		group_data.group_data.app_id.as_str() +
		"_" + group_data.group_data.id.as_str() +
		"_" + group_data.user_data.user_id.as_str();

	cache::delete(key_group.as_str()).await;

	echo_success()
}

//__________________________________________________________________________________________________

async fn insert_user_keys_via_session(mut req: Request, insert_type: GroupNewUserType) -> JRes<ServerSuccessOutput>
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

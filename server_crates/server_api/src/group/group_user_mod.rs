use rustgram::Request;
use sentc_crypto_common::group::GroupKeysForNewMemberServerInput;
use sentc_crypto_common::server_default::ServerSuccessOutput;

use crate::core::api_res::{echo_success, ApiErrorCodes, HttpErr, JRes};
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};
use crate::group::group_user_model;
use crate::user::jwt::get_jwt_data_from_param;

pub(crate) async fn invite_request(mut req: Request) -> JRes<ServerSuccessOutput>
{
	//no the accept invite, but the keys are prepared for the invited user
	//don't save this values in the group user keys table, but in the invite table

	let body = get_raw_body(&mut req).await?;

	let user = get_jwt_data_from_param(&req)?;
	let req_params = get_params(&req)?;
	let group_id = get_name_param_from_params(req_params, "group_id")?;
	let invited_user = get_name_param_from_params(req_params, "invited_user")?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	if input.0.len() == 0 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupNoKeys,
			"No group keys for the user".to_string(),
			None,
		));
	}

	group_user_model::invite_request(
		user.sub.to_string(),
		group_id.to_string(),
		user.id.to_string(),
		invited_user.to_string(),
		input.0,
	)
	.await?;

	echo_success()
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

pub(crate) async fn leave_group(req: Request) -> JRes<ServerSuccessOutput>
{
	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	group_user_model::user_leave_group(user.sub.to_string(), group_id.to_string(), user.id.to_string()).await?;

	echo_success()
}

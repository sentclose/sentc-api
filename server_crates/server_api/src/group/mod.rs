mod group_entities;
mod group_model;

use rustgram::Request;
use sentc_crypto_common::group::{CreateData, GroupCreateOutput, GroupDeleteServerOutput, GroupKeysForNewMemberServerInput};
use sentc_crypto_common::server_default::ServerSuccessOutput;

use crate::core::api_res::{echo, echo_success, HttpErr, JRes};
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};
use crate::user::jwt::get_jwt_data_from_param;

pub(crate) async fn create(mut req: Request) -> JRes<GroupCreateOutput>
{
	let body = get_raw_body(&mut req).await?;

	let user = get_jwt_data_from_param(&req)?;

	let input: CreateData = bytes_to_json(&body)?;

	let group_id = group_model::create(user.sub.to_string(), user.id.to_string(), input).await?;

	let out = GroupCreateOutput {
		group_id,
	};

	echo(out)
}

pub(crate) async fn delete(req: Request) -> JRes<GroupDeleteServerOutput>
{
	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	group_model::delete(user.sub.to_string(), group_id.to_string(), user.id.to_string()).await?;

	let out = GroupDeleteServerOutput {
		group_id: group_id.to_string(),
	};

	echo(out)
}

pub async fn invite_request(mut req: Request) -> JRes<ServerSuccessOutput>
{
	//no the accept invite, but the keys are prepared for the invited user
	//don't save this values in the group user keys table, but in the invite table

	let body = get_raw_body(&mut req).await?;

	let user = get_jwt_data_from_param(&req)?;
	let req_params = get_params(&req)?;
	let group_id = get_name_param_from_params(req_params, "group_id")?;
	let invited_user = get_name_param_from_params(req_params, "invited_user")?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	group_model::invite_request(
		group_id.to_string(),
		user.id.to_string(),
		invited_user.to_string(),
		input.0,
	)
	.await?;

	echo_success()
}

pub(crate) async fn get(_req: Request) -> Result<String, HttpErr>
{
	Ok(format!("group"))
}

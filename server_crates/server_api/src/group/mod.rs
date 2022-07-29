mod group_entities;
mod group_model;

use rustgram::Request;
use sentc_crypto_common::group::{
	CreateData,
	GroupCreateOutput,
	GroupDeleteServerOutput,
	GroupKeyServerOutput,
	GroupKeysForNewMemberServerInput,
	GroupServerData,
};
use sentc_crypto_common::server_default::ServerSuccessOutput;

use crate::core::api_res::{echo, echo_success, ApiErrorCodes, HttpErr, JRes};
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

	group_model::invite_request(
		group_id.to_string(),
		user.id.to_string(),
		invited_user.to_string(),
		input.0,
	)
	.await?;

	echo_success()
}

pub(crate) async fn get_user_group_data(req: Request) -> JRes<GroupServerData>
{
	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	let (user_group_data, user_keys) = group_model::get_user_group_data(user.sub.to_string(), user.id.to_string(), group_id.to_string()).await?;

	let mut keys: Vec<GroupKeyServerOutput> = Vec::with_capacity(user_keys.len());

	for user_key in user_keys {
		keys.push(user_key.into());
	}

	let key_update = group_model::check_for_key_update(user.sub.to_string(), user.sub.to_string(), group_id.to_string()).await?;

	let out = GroupServerData {
		group_id: user_group_data.id,
		parent_group_id: user_group_data.parent_group_id,
		keys,
		key_update,
		rank: user_group_data.rank,
		created_time: user_group_data.created_time,
		joined_time: user_group_data.joined_time,
	};

	echo(out)
}

pub(crate) async fn get_user_group_keys(req: Request) -> JRes<Vec<GroupKeyServerOutput>>
{
	let user = get_jwt_data_from_param(&req)?;
	let req_params = get_params(&req)?;
	let group_id = get_name_param_from_params(req_params, "group_id")?;

	let last_fetched_time = get_name_param_from_params(req_params, "last_fetched_time")?;
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
	)
	.await?;

	let mut keys: Vec<GroupKeyServerOutput> = Vec::with_capacity(user_keys.len());
	for user_key in user_keys {
		keys.push(user_key.into());
	}

	echo(keys)
}

pub(crate) async fn get(_req: Request) -> Result<String, HttpErr>
{
	Ok(format!("group"))
}

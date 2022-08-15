mod group_key_rotation_model;
pub(crate) mod group_key_rotation_worker;

use rustgram::Request;
use sentc_crypto_common::group::{DoneKeyRotationData, KeyRotationData, KeyRotationInput, KeyRotationStartServerOutput};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use server_core::input_helper::{bytes_to_json, get_raw_body};
use server_core::url_helper::get_name_param_from_req;

use crate::customer_app::app_util::{check_endpoint_with_req, Endpoint};
use crate::group::get_group_user_data_from_req;
use crate::util::api_res::{echo, echo_success, JRes};

pub(crate) async fn start_key_rotation(mut req: Request) -> JRes<KeyRotationStartServerOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupKeyRotation)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let input: KeyRotationData = bytes_to_json(&body)?;

	let key_id = group_key_rotation_model::start_key_rotation(
		group_data.group_data.app_id.to_string(),
		group_data.group_data.id.to_string(),
		group_data.user_data.user_id.to_string(),
		input,
	)
	.await?;

	//TODO change this to a real worker after beta
	group_key_rotation_worker::start(group_data.group_data.id.to_string(), key_id.to_string()).await?;

	let out = KeyRotationStartServerOutput {
		key_id,
		group_id: group_data.group_data.id.to_string(),
	};

	echo(out)
}

pub(crate) async fn get_keys_for_update(req: Request) -> JRes<Vec<KeyRotationInput>>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDataGet)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let update = group_key_rotation_model::get_keys_for_key_update(
		group_data.group_data.app_id.to_string(),
		group_data.group_data.id.to_string(),
		group_data.user_data.user_id.to_string(),
	)
	.await?;

	let mut out: Vec<KeyRotationInput> = Vec::with_capacity(update.len());

	for keys in update {
		out.push(keys.into());
	}

	echo(out)
}

pub(crate) async fn done_key_rotation_for_user(mut req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDataGet)?;

	//called from the user
	let body = get_raw_body(&mut req).await?;

	let group_data = get_group_user_data_from_req(&req)?;
	let key_id = get_name_param_from_req(&req, "key_id")?;

	let input: DoneKeyRotationData = bytes_to_json(&body)?;

	group_key_rotation_model::done_key_rotation_for_user(
		group_data.group_data.id.to_string(),
		group_data.user_data.user_id.to_string(),
		key_id.to_string(),
		input,
	)
	.await?;

	echo_success()
}

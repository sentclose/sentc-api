use rustgram::Request;
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, JRes, ServerSuccessOutput};
use rustgram_server_util::url_helper::get_name_param_from_req;
use sentc_crypto_common::group::{DoneKeyRotationData, KeyRotationData, KeyRotationStartServerOutput};

use crate::customer_app::app_util::{check_endpoint_with_req, Endpoint};
use crate::group::get_group_user_data_from_req;
use crate::group::group_entities::GroupKeyUpdate;
use crate::group::group_key_rotation::group_key_rotation_service;

pub async fn start_key_rotation(mut req: Request) -> JRes<KeyRotationStartServerOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::GroupKeyRotation)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let input: KeyRotationData = bytes_to_json(&body)?;

	let out = group_key_rotation_service::start_key_rotation(
		&group_data.group_data.app_id,
		&group_data.group_data.id,
		&group_data.user_data.user_id,
		input,
		None,
	)
	.await?;

	echo(out)
}

pub async fn get_keys_for_update(req: Request) -> JRes<Vec<GroupKeyUpdate>>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDataGet)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let update = group_key_rotation_service::get_keys_for_update(
		&group_data.group_data.app_id,
		&group_data.group_data.id,
		&group_data.user_data.user_id,
	)
	.await?;

	echo(update)
}

pub async fn done_key_rotation_for_user(mut req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::GroupUserDataGet)?;

	//called from the user
	let body = get_raw_body(&mut req).await?;

	let group_data = get_group_user_data_from_req(&req)?;
	let key_id = get_name_param_from_req(&req, "key_id")?;

	let input: DoneKeyRotationData = bytes_to_json(&body)?;

	group_key_rotation_service::done_key_rotation_for_user(
		&group_data.group_data.id,
		&group_data.user_data.user_id,
		key_id,
		input,
	)
	.await?;

	echo_success()
}

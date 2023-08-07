use rustgram::Request;
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, JRes, ServerSuccessOutput};
use rustgram_server_util::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params, get_time_from_url_param};
use sentc_crypto_common::crypto::{GeneratedSymKeyHeadServerInput, GeneratedSymKeyHeadServerRegisterOutput};
use server_api_common::customer_app::{check_endpoint_with_app_options, get_app_data_from_req, Endpoint};
use server_api_common::user::get_jwt_data_from_param;

use crate::key_management::key_entity::SymKeyEntity;
use crate::key_management::key_model;

pub async fn register_sym_key(mut req: Request) -> JRes<GeneratedSymKeyHeadServerRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: GeneratedSymKeyHeadServerInput = bytes_to_json(&body)?;

	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::KeyRegister)?;
	let user = get_jwt_data_from_param(&req)?;

	let key_id = key_model::register_sym_key(&app.app_data.app_id, &user.id, input).await?;

	let out = GeneratedSymKeyHeadServerRegisterOutput {
		key_id,
	};

	echo(out)
}

pub async fn delete_sym_key(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::KeyRegister)?;

	let user = get_jwt_data_from_param(&req)?;

	let key_id = get_name_param_from_req(&req, "key_id")?;

	key_model::delete_sym_key(&app.app_data.app_id, &user.id, key_id).await?;

	echo_success()
}

pub async fn get_sym_key_by_id(req: Request) -> JRes<SymKeyEntity>
{
	let app_data = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app_data, Endpoint::KeyGet)?;

	let key_id = get_name_param_from_req(&req, "key_id")?;

	let key = key_model::get_sym_key_by_id(&app_data.app_data.app_id, key_id).await?;

	echo(key)
}

pub async fn get_all_sym_keys_to_master_key(req: Request) -> JRes<Vec<SymKeyEntity>>
{
	let app_data = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app_data, Endpoint::KeyGet)?;

	let params = get_params(&req)?;

	let master_key_id = get_name_param_from_params(params, "master_key_id")?;
	let last_key_id = get_name_param_from_params(params, "last_key_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let keys = key_model::get_all_sym_keys_to_master_key(
		&app_data.app_data.app_id,
		master_key_id,
		last_fetched_time,
		last_key_id,
	)
	.await?;

	echo(keys)
}

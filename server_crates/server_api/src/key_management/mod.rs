use rustgram::Request;
use sentc_crypto_common::crypto::{GeneratedSymKeyHeadServerInput, GeneratedSymKeyHeadServerOutput, GeneratedSymKeyHeadServerRegisterOutput};
use sentc_crypto_common::server_default::ServerSuccessOutput;

use crate::core::api_res::{echo, echo_success, ApiErrorCodes, HttpErr, JRes};
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};
use crate::customer_app::app_util::{check_endpoint_with_app_options, check_endpoint_with_req, get_app_data_from_req, Endpoint};
use crate::user::jwt::get_jwt_data_from_param;

mod key_entity;
mod key_model;

pub(crate) async fn register_sym_key(mut req: Request) -> JRes<GeneratedSymKeyHeadServerRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: GeneratedSymKeyHeadServerInput = bytes_to_json(&body)?;

	check_endpoint_with_req(&req, Endpoint::KeyRegister)?;
	let user = get_jwt_data_from_param(&req)?;

	let key_id = key_model::register_sym_key(user.sub.to_string(), user.id.to_string(), input).await?;

	let out = GeneratedSymKeyHeadServerRegisterOutput {
		key_id,
	};

	echo(out)
}

pub(crate) async fn delete_sym_key(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::KeyRegister)?;
	let user = get_jwt_data_from_param(&req)?;

	let key_id = get_name_param_from_req(&req, "key_id")?;

	key_model::delete_sym_key(user.sub.to_string(), user.id.to_string(), key_id.to_string()).await?;

	echo_success()
}

pub(crate) async fn get_sym_key_by_id(req: Request) -> JRes<GeneratedSymKeyHeadServerOutput>
{
	let app_data = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app_data, Endpoint::KeyGet)?;

	let key_id = get_name_param_from_req(&req, "key_id")?;

	let key = key_model::get_sym_key_by_id(app_data.app_data.app_id.to_string(), key_id.to_string()).await?;

	echo(key.into())
}

pub(crate) async fn get_all_sym_keys_to_master_key(req: Request) -> JRes<Vec<GeneratedSymKeyHeadServerOutput>>
{
	let app_data = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app_data, Endpoint::KeyGet)?;

	let params = get_params(&req)?;

	let master_key_id = get_name_param_from_params(&params, "master_key_id")?;
	let last_key_id = get_name_param_from_params(&params, "last_key_id")?;
	let last_fetched_time = get_name_param_from_params(&params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let keys = key_model::get_all_sym_keys_to_master_key(
		app_data.app_data.app_id.to_string(),
		master_key_id.to_string(),
		last_fetched_time,
		last_key_id.to_string(),
	)
	.await?;

	let mut out: Vec<GeneratedSymKeyHeadServerOutput> = Vec::with_capacity(keys.len());
	for key in keys {
		out.push(key.into());
	}

	echo(out)
}

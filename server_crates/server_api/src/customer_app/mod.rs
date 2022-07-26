pub(crate) mod app_entities;
pub(crate) mod app_model;
pub(crate) mod app_util;

use rand::RngCore;
use rustgram::Request;

use crate::core::api_res::{echo, ApiErrorCodes, HttpErr, JRes};
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::customer_app::app_entities::{AppJwtRegisterOutput, AppRegisterInput, AppRegisterOutput};
use crate::customer_app::app_util::{hash_token_to_string, HASH_ALG};
use crate::user::jwt::create_jwt_keys;

pub(crate) async fn create_app(mut req: Request) -> JRes<AppRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: AppRegisterInput = bytes_to_json(&body)?;

	//TODO activate it when customer mod is done
	// let customer = get_jwt_data_from_param(&req)?;
	// let customer_id = &customer.id;

	let customer_id = &"abcdefg".to_string();

	//1. create and hash tokens
	let (secret_token, public_token) = generate_tokens()?;

	let hashed_secret_token = hash_token_to_string(&secret_token)?;
	let hashed_public_token = hash_token_to_string(&public_token)?;

	//2. create the first jwt keys
	let (jwt_sign_key, jwt_verify_key, alg) = create_jwt_keys()?;

	//3. create an new app (with new secret_token and public_token)
	//	the str values are used because the real values are exported to the user
	let app_id = app_model::create_app(
		customer_id,
		input.identifier,
		hashed_secret_token,
		hashed_public_token,
		HASH_ALG,
		jwt_sign_key.as_str(),
		jwt_verify_key.as_str(),
		alg,
	)
	.await?;

	let customer_app_data = AppRegisterOutput {
		customer_id: customer_id.to_string(),
		app_id: app_id.to_string(),
		secret_token: base64::encode(secret_token),
		public_token: base64::encode(public_token),
		jwt_data: AppJwtRegisterOutput {
			customer_id: customer_id.to_string(),
			app_id,
			jwt_verify_key,
			jwt_sign_key,
			jwt_alg: alg.to_string(),
		},
	};

	echo(customer_app_data)
}

fn generate_tokens() -> Result<([u8; 50], [u8; 30]), HttpErr>
{
	let mut rng = rand::thread_rng();

	let mut secret_token = [0u8; 50];

	rng.try_fill_bytes(&mut secret_token).map_err(|_| {
		HttpErr::new(
			400,
			ApiErrorCodes::AppTokenWrongFormat,
			"Can't create secret token".to_string(),
			None,
		)
	})?;

	let mut public_token = [0u8; 30];

	rng.try_fill_bytes(&mut public_token).map_err(|_| {
		HttpErr::new(
			400,
			ApiErrorCodes::AppTokenWrongFormat,
			"Can't create secret token".to_string(),
			None,
		)
	})?;

	Ok((secret_token, public_token))
}

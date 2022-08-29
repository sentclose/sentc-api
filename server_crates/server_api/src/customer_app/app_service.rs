use sentc_crypto_common::CustomerId;
use server_api_common::app::{AppFileOptions, AppJwtRegisterOutput, AppRegisterInput, AppRegisterOutput, FILE_STORAGE_OWN};

use crate::customer_app::app_util::{hash_token_to_string, HASH_ALG};
use crate::customer_app::{app_model, generate_tokens};
use crate::user::jwt::create_jwt_keys;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

pub async fn create_app(input: AppRegisterInput, customer_id: CustomerId) -> AppRes<AppRegisterOutput>
{
	//1. create and hash tokens
	let (secret_token, public_token) = generate_tokens()?;

	let hashed_secret_token = hash_token_to_string(&secret_token)?;
	let hashed_public_token = hash_token_to_string(&public_token)?;

	//2. create the first jwt keys
	let (jwt_sign_key, jwt_verify_key, alg) = create_jwt_keys()?;

	check_file_options(&input.file_options)?;

	//3. create an new app (with new secret_token and public_token)
	//	the str values are used because the real values are exported to the user
	let (app_id, jwt_id) = app_model::create_app(
		&customer_id,
		input,
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
			customer_id,
			app_id,
			jwt_id,
			jwt_verify_key,
			jwt_sign_key,
			jwt_alg: alg.to_string(),
		},
	};

	Ok(customer_app_data)
}

pub(super) fn check_file_options(input: &AppFileOptions) -> AppRes<()>
{
	//check the file option if the right storage is used
	if input.file_storage > 1 || input.file_storage < -1 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::AppAction,
			"Wrong storage selected".to_string(),
			None,
		));
	}

	if input.file_storage == FILE_STORAGE_OWN && input.storage_url == "" {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::AppAction,
			"No external storage selected for files".to_string(),
			None,
		));
	}

	Ok(())
}

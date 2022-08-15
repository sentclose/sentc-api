use sentc_crypto_common::CustomerId;
use server_api_common::app::{AppJwtRegisterOutput, AppRegisterInput, AppRegisterOutput};

use crate::customer_app::app_util::{hash_token_to_string, HASH_ALG};
use crate::customer_app::{app_model, generate_tokens};
use crate::user::jwt::create_jwt_keys;
use crate::util::api_res::AppRes;

pub async fn create_app(input: AppRegisterInput, customer_id: CustomerId) -> AppRes<AppRegisterOutput>
{
	//1. create and hash tokens
	let (secret_token, public_token) = generate_tokens()?;

	let hashed_secret_token = hash_token_to_string(&secret_token)?;
	let hashed_public_token = hash_token_to_string(&public_token)?;

	//2. create the first jwt keys
	let (jwt_sign_key, jwt_verify_key, alg) = create_jwt_keys()?;

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

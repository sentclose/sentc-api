pub mod jwt;
pub(crate) mod user_entities;
mod user_model;

use rustgram::Request;
use sentc_crypto_common::user::{
	DoneLoginServerInput,
	DoneLoginServerKeysOutput,
	PrepareLoginSaltServerOutput,
	PrepareLoginServerInput,
	RegisterData,
	UserIdentifierAvailableServerOutput,
};

use crate::core::api_res::HttpErr;
use crate::core::input_helper::{bytes_to_json, get_raw_body, json_to_string};

pub(crate) async fn exists(_req: Request) -> Result<String, HttpErr>
{
	let user_id = "058ed2e6-3880-4a7c-ab3b-fd2f5755ea43"; //get this from the url param

	let exists = user_model::check_user_exists(user_id).await?;

	let out = UserIdentifierAvailableServerOutput {
		user_identifier: user_id.to_string(),
		available: exists,
	};

	json_to_string(&out)
}

pub(crate) async fn register(req: Request) -> Result<String, HttpErr>
{
	//load the register input from the req body
	let body = get_raw_body(req).await?;

	let _register_input: RegisterData = bytes_to_json(&body)?;

	Ok(format!("done"))
}

pub(crate) async fn prepare_login(req: Request) -> Result<String, HttpErr>
{
	let body = get_raw_body(req).await?;

	let _user_identifier: PrepareLoginServerInput = bytes_to_json(&body)?;

	//check the user id in the db

	//create the salt

	json_to_string(&PrepareLoginSaltServerOutput {
		salt_string: "".to_string(),
		derived_encryption_key_alg: "".to_string(),
	})
}

pub(crate) async fn done_login(req: Request) -> Result<String, HttpErr>
{
	let body = get_raw_body(req).await?;

	let _done_login: DoneLoginServerInput = bytes_to_json(&body)?;

	//hash the auth key and use the first 16 bytes

	//check the hashed auth key in the db

	//if not correct -> err msg

	//if correct -> fetch and return the user data
	// and create the jwt

	json_to_string(&DoneLoginServerKeysOutput {
		encrypted_master_key: "".to_string(),
		encrypted_private_key: "".to_string(),
		public_key_string: "".to_string(),
		keypair_encrypt_alg: "".to_string(),
		encrypted_sign_key: "".to_string(),
		verify_key_string: "".to_string(),
		keypair_sign_alg: "".to_string(),
		keypair_encrypt_id: "".to_string(),
		keypair_sign_id: "".to_string(),
		jwt: "".to_string(),
	})
}

pub(crate) async fn get(_req: Request) -> Result<String, HttpErr>
{
	let user_id = "abc"; //get this from the url param

	//
	let user = user_model::get_user(user_id).await?;

	json_to_string(&user)
}

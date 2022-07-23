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

use crate::core::api_res::{echo, HttpErr, JRes};
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::user::user_entities::UserEntity;

pub(crate) async fn exists(_req: Request) -> JRes<UserIdentifierAvailableServerOutput>
{
	let user_id = "058ed2e6-3880-4a7c-ab3b-fd2f5755ea43"; //get this from the url param

	let exists = user_model::check_user_exists(user_id).await?;

	let out = UserIdentifierAvailableServerOutput {
		user_identifier: user_id.to_string(),
		available: exists,
	};

	echo(out)
}

pub(crate) async fn register(mut req: Request) -> Result<String, HttpErr>
{
	//load the register input from the req body
	let body = get_raw_body(&mut req).await?;

	let _register_input: RegisterData = bytes_to_json(&body)?;

	Ok(format!("done"))
}

pub(crate) async fn prepare_login(mut req: Request) -> JRes<PrepareLoginSaltServerOutput>
{
	let body = get_raw_body(&mut req).await?;

	let _user_identifier: PrepareLoginServerInput = bytes_to_json(&body)?;

	//check the user id in the db

	//create the salt

	let out = PrepareLoginSaltServerOutput {
		salt_string: "".to_string(),
		derived_encryption_key_alg: "".to_string(),
	};

	echo(out)
}

pub(crate) async fn done_login(mut req: Request) -> JRes<DoneLoginServerKeysOutput>
{
	let body = get_raw_body(&mut req).await?;

	let _done_login: DoneLoginServerInput = bytes_to_json(&body)?;

	//hash the auth key and use the first 16 bytes

	//check the hashed auth key in the db

	//if not correct -> err msg

	//if correct -> fetch and return the user data
	// and create the jwt

	let out = DoneLoginServerKeysOutput {
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
	};

	echo(out)
}

pub(crate) async fn get(_req: Request) -> JRes<UserEntity>
{
	let user_id = "abc"; //get this from the url param

	//
	let user = user_model::get_user(user_id).await?;

	echo(user)
}

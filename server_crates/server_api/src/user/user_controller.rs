use rustgram::Request;
use sentc_crypto_common::user::{RegisterData, UserIdentifierAvailableServerOutput};

use crate::core::api_err::HttpErr;
use crate::core::input_helper::{bytes_to_json, get_raw_body, json_to_string};
use crate::user::user_model;

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

pub(crate) async fn get(_req: Request) -> Result<String, HttpErr>
{
	let user_id = "abc"; //get this from the url param

	//
	let user = user_model::get_user(user_id).await?;

	json_to_string(&user)
}

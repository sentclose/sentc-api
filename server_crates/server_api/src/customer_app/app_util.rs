use ring::digest::{Context, SHA256};
use rustgram::Request;

use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::customer_app::app_entities::AppData;

pub static HASH_ALG: &'static str = "SHA256";

pub enum Endpoint
{
	UserExistsCheck,
	UserRegister,
	UserDelete,
	UserPrepLogin,
	UserDoneLogin,

	GroupCreate,
	GroupDelete,
}

pub(crate) fn get_app_data_from_req(req: &Request) -> AppRes<&AppData>
{
	//this should always be there because it is checked in the app token mw
	match req.extensions().get::<AppData>() {
		Some(e) => Ok(e),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::AppNotFound,
				"No app found".to_string(),
				None,
			))
		},
	}
}

pub fn hash_token(token: &[u8]) -> AppRes<[u8; 32]>
{
	let mut context = Context::new(&SHA256);
	context.update(token);
	let result = context.finish();

	let hashed_token: [u8; 32] = result.as_ref().try_into().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::AppTokenWrongFormat,
			"Token can't be hashed".to_string(),
			None,
		)
	})?;

	Ok(hashed_token)
}

pub fn hash_token_to_string(token: &[u8]) -> AppRes<String>
{
	let token = hash_token(&token)?;

	Ok(base64::encode(token))
}

pub fn hash_token_from_string_to_string(token: &str) -> AppRes<String>
{
	//the normal token is also encoded as base64 when exporting it to user
	let token = base64::decode(token).map_err(|_e| {
		HttpErr::new(
			401,
			ApiErrorCodes::AppTokenWrongFormat,
			"Token can't be hashed".to_string(),
			None,
		)
	})?;

	hash_token_to_string(&token)
}

pub fn check_endpoint_with_app_options(app_data: &AppData, endpoint: Endpoint) -> AppRes<()>
{
	todo!()
}

/**
Check the endpoint with the app options

get the options from req
*/
pub fn check_endpoint_with_req(req: &Request, endpoint: Endpoint) -> AppRes<()>
{
	let data = get_app_data_from_req(req)?;

	check_endpoint_with_app_options(data, endpoint)
}

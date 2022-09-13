use alloc::string::String;

use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto_full::util::{make_req, HttpMethod};
use server_api_common::app::{AppFileOptions, AppOptions, AppRegisterInput, AppUpdateInput};

use crate::utils;

pub async fn create(
	base_url: String,
	auth_token: &str,
	jwt: &str,
	identifier: Option<String>,
	options: AppOptions,
	file_options: AppFileOptions,
) -> Result<server_api_common::app::AppRegisterOutput, String>
{
	let input = AppRegisterInput {
		identifier,
		options,
		file_options,
	};
	let input = utils::to_string(&input)?;

	let url = base_url + "/api/v1/customer/app";

	let res = make_req(HttpMethod::POST, url.as_str(), auth_token, Some(input), Some(jwt)).await?;

	let out: server_api_common::app::AppRegisterOutput = handle_server_response(res.as_str())?;

	Ok(out)
}

pub async fn update(base_url: String, auth_token: &str, jwt: &str, app_id: &str, identifier: Option<String>) -> Result<(), String>
{
	let input = AppUpdateInput {
		identifier,
	};
	let input = utils::to_string(&input)?;

	let url = base_url + "/api/v1/customer/app/" + app_id;

	let res = make_req(HttpMethod::PUT, url.as_str(), auth_token, Some(input), Some(jwt)).await?;

	Ok(handle_general_server_response(res.as_str())?)
}

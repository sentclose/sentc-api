use alloc::string::String;

use sentc_crypto::util::public::handle_server_response;
use sentc_crypto::SdkError;
use sentc_crypto_full::util::{make_req, HttpMethod};
use server_api_common::app::{AppFileOptions, AppOptions, AppRegisterInput};

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
	let input = serde_json::to_string(&input).map_err(|_| SdkError::JsonToStringFailed)?;

	let url = base_url + "/api/v1/customer/app";

	let res = make_req(HttpMethod::POST, url.as_str(), auth_token, Some(input), Some(jwt)).await?;

	let out: server_api_common::app::AppRegisterOutput = handle_server_response(res.as_str())?;

	Ok(out)
}

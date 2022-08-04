use std::env;

use sentc_crypto_common::{AppId, CustomerId, JwtKeyId, ServerOutput};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

#[derive(Serialize, Deserialize)]
pub struct AppRegisterInput
{
	pub identifier: Option<String>,
}

impl AppRegisterInput
{
	pub fn to_string(&self) -> serde_json::Result<String>
	{
		to_string(self)
	}
}

#[derive(Serialize, Deserialize)]
pub struct AppJwtRegisterOutput
{
	pub customer_id: CustomerId,
	pub app_id: AppId,
	pub jwt_id: JwtKeyId,
	pub jwt_verify_key: String,
	pub jwt_sign_key: String,
	pub jwt_alg: String, //should be ES384 for now
}

#[derive(Serialize, Deserialize)]
pub struct AppRegisterOutput
{
	pub customer_id: CustomerId,
	pub app_id: AppId,

	//don't show this values in te normal app data
	pub secret_token: String,
	pub public_token: String,

	pub jwt_data: AppJwtRegisterOutput,
}

/**
Creates new tokens for the base sentc app, to manage the customer mod.

Only works with running api server.
*/
fn main()
{
	//load the env
	dotenv::dotenv().ok();

	let input = AppRegisterInput {
		identifier: None,
	};

	let url = env::var("PUBLIC_URL").unwrap() + "/api/v1/customer/app";

	let client = reqwest::blocking::Client::new();
	let res = client
		.post(url)
		.body(input.to_string().unwrap())
		.send()
		.unwrap();

	let body = res.text().unwrap();

	let out = ServerOutput::<AppRegisterOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let app_data = out.result.unwrap();

	println!("secret_token: {}", app_data.secret_token);
	println!("public_token: {}", app_data.public_token);
}

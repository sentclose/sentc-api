use std::env;

use sentc_crypto_common::ServerOutput;
use server_api_common::app::{AppRegisterInput, AppRegisterOutput};

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

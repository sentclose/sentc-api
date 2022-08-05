use std::env;

use sentc_crypto_common::user::RegisterData;
use sentc_crypto_common::ServerOutput;
use server_api::core::api_res::ApiErrorCodes;
use server_api_common::customer::{CustomerRegisterData, CustomerRegisterOutput};

use crate::test_fn::get_url;

mod test_fn;

#[ignore]
#[tokio::test]
async fn test_0_register_customer_with_email()
{
	//Test here with real email send -> don't do it in regular test run!

	let url = get_url("api/v1/customer/register".to_string());

	dotenv::dotenv().ok();

	let email = env::var("EMAIL_ADDRESS_TEST").unwrap();

	let register_data = sentc_crypto::user::register(email.as_str(), "12345").unwrap();
	let register_data = RegisterData::from_string(register_data.as_str()).unwrap();

	let input = CustomerRegisterData {
		email,
		register_data,
	};

	let public_token = env::var("SENTC_PUBLIC_TOKEN").unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	println!("result for registration = {}", body);

	let out = ServerOutput::<CustomerRegisterOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);
}

#[tokio::test]
async fn aaa_init_global()
{
	dotenv::dotenv().ok();
}

#[tokio::test]
async fn test_10_register_without_valid_email()
{
	let url = get_url("api/v1/customer/register".to_string());

	let wrong_email = "hello@localhost".to_string();

	let register_data = sentc_crypto::user::register(wrong_email.as_str(), "12345").unwrap();
	let register_data = RegisterData::from_string(register_data.as_str()).unwrap();

	let input = CustomerRegisterData {
		email: wrong_email,
		register_data,
	};

	let public_token = env::var("SENTC_PUBLIC_TOKEN").unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<CustomerRegisterOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, false);
	assert_eq!(
		out.err_code.unwrap(),
		ApiErrorCodes::CustomerEmailSyntax.get_int_code()
	);
}

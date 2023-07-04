use std::env;

use rustgram_server_util::error::ServerErrorCodes;
use sentc_crypto_common::user::RegisterData;
use sentc_crypto_common::ServerOutput;
use server_api::util::api_res::ApiErrorCodes;
use server_api_common::customer::{CustomerData, CustomerRegisterData, CustomerRegisterOutput};

use crate::test_fn::{get_captcha, get_url};

mod test_fn;

#[ignore]
#[tokio::test]
async fn test_0_register_customer_with_email()
{
	//Test here with real email send -> don't do it in regular test run!

	let url = get_url("api/v1/customer/register".to_string());

	dotenv::from_filename("sentc.env").ok();

	let email = env::var("EMAIL_ADDRESS_TEST").unwrap();

	let register_data = sentc_crypto::user::register(email.as_str(), "12345").unwrap();
	let register_data = RegisterData::from_string(register_data.as_str()).unwrap();

	let captcha_input = get_captcha().await;

	let input = CustomerRegisterData {
		customer_data: CustomerData {
			name: "abc".to_string(),
			first_name: "abc".to_string(),
			company: None,
		},
		email,
		register_data: register_data.device,
		captcha_input,
	};

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	println!("result for registration = {}", body);

	let out = ServerOutput::<CustomerRegisterOutput>::from_string(body.as_str()).unwrap();

	assert!(out.status);
	assert_eq!(out.err_code, None);
}

#[ignore]
#[tokio::test]
async fn test_10_1_not_register_when_register_is_disabled()
{
	//only run with env CUSTOMER_REGISTER=0

	let url = get_url("api/v1/customer/register".to_string());

	let email = "hello@localhost.de".to_string();

	let register_data = sentc_crypto::user::register(email.as_str(), "12345").unwrap();
	let register_data = RegisterData::from_string(register_data.as_str()).unwrap();

	let captcha_input = get_captcha().await;

	let input = CustomerRegisterData {
		customer_data: CustomerData {
			name: "abc".to_string(),
			first_name: "abc".to_string(),
			company: None,
		},
		email,
		register_data: register_data.device,
		captcha_input,
	};

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<CustomerRegisterOutput>::from_string(body.as_str()).unwrap();

	assert!(!out.status);
	assert_eq!(out.err_code.unwrap(), ApiErrorCodes::CustomerDisable.get_int_code());
}

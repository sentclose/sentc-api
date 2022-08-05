use std::env;

use reqwest::header::AUTHORIZATION;
use sentc_crypto_common::user::RegisterData;
use sentc_crypto_common::ServerOutput;
use server_api::core::api_res::ApiErrorCodes;
use server_api_common::customer::{CustomerDoneLoginOutput, CustomerRegisterData, CustomerRegisterOutput, CustomerUpdateInput};
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{auth_header, get_url, login_customer};

mod test_fn;

pub struct CustomerState
{
	pub public_token: String,
	pub customer_id: String,
	pub customer_email: String,
	pub customer_pw: String,
	pub customer_data: Option<CustomerDoneLoginOutput>,
}

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerState>> = OnceCell::const_new();

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

	let public_token = env::var("SENTC_PUBLIC_TOKEN").unwrap();

	CUSTOMER_TEST_STATE
		.get_or_init(|| {
			async move {
				//
				RwLock::new(CustomerState {
					public_token,
					customer_id: "".to_string(),
					customer_email: "".to_string(),
					customer_pw: "".to_string(),
					customer_data: None,
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_10_register_without_valid_email()
{
	let customer = CUSTOMER_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/customer/register".to_string());

	let wrong_email = "hello@localhost".to_string();

	let register_data = sentc_crypto::user::register(wrong_email.as_str(), "12345").unwrap();
	let register_data = RegisterData::from_string(register_data.as_str()).unwrap();

	let input = CustomerRegisterData {
		email: wrong_email,
		register_data,
	};

	let public_token = customer.public_token.as_str();

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

#[tokio::test]
async fn test_11_register_customer()
{
	let mut customer = CUSTOMER_TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/customer/register".to_string());

	let email = "hello@test.com".to_string();

	let register_data = sentc_crypto::user::register(email.as_str(), "12345").unwrap();
	let register_data = RegisterData::from_string(register_data.as_str()).unwrap();

	let input = CustomerRegisterData {
		email: email.to_string(),
		register_data,
	};

	let public_token = customer.public_token.as_str();

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

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	customer.customer_id = out.customer_id;
	customer.customer_email = email;
	customer.customer_pw = "12345".to_string();
}

#[tokio::test]
async fn test_12_login_customer()
{
	let mut customer = CUSTOMER_TEST_STATE.get().unwrap().write().await;

	//login customer

	let email = &customer.customer_email;
	let pw = &customer.customer_pw;
	let public_token = customer.public_token.as_str();

	let url = get_url("api/v1/customer/prepare_login".to_owned());

	let prep_server_input = sentc_crypto::user::prepare_login_start(email).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (auth_key, _derived_master_key) = sentc_crypto::user::prepare_login(email, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/customer/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
		.body(auth_key)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<CustomerDoneLoginOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let login_data = out.result.unwrap();

	customer.customer_data = Some(login_data);
}

#[tokio::test]
async fn test_13_update_customer()
{
	let mut customer = CUSTOMER_TEST_STATE.get().unwrap().write().await;

	let email = &customer.customer_email;
	let pw = &customer.customer_pw;
	let public_token = customer.public_token.as_str();
	let jwt = &customer.customer_data.as_ref().unwrap().user_keys.jwt;

	let new_email = "hello3@test.com".to_string();

	let update_data = CustomerUpdateInput {
		new_email: new_email.to_string(),
	};

	let url = get_url("api/v1/customer".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", public_token)
		.header(AUTHORIZATION, auth_header(jwt))
		.body(serde_json::to_string(&update_data).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util_pub::handle_general_server_response(body.as_str()).unwrap();

	//______________________________________________________________________________________________
	//it should not login with the old email

	let url = get_url("api/v1/customer/prepare_login".to_owned());

	let prep_server_input = sentc_crypto::user::prepare_login_start(email).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (auth_key, _derived_master_key) = sentc_crypto::user::prepare_login(email, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/customer/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
		.body(auth_key)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<CustomerDoneLoginOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, false);

	//______________________________________________________________________________________________
	//login with the right data

	let login_data = login_customer(new_email.as_str(), pw).await;

	customer.customer_data = Some(login_data);
	customer.customer_email = new_email;
}

//__________________________________________________________________________________________________

#[tokio::test]
async fn test_30_delete_customer()
{
	let customer = CUSTOMER_TEST_STATE.get().unwrap().read().await;

	let public_token = &customer.public_token;
	let jwt = &customer.customer_data.as_ref().unwrap().user_keys.jwt;

	let url = get_url("api/v1/customer".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header("x-sentc-app-token", public_token.as_str())
		.header(AUTHORIZATION, auth_header(jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util_pub::handle_general_server_response(body.as_str()).unwrap();
}

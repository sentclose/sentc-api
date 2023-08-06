use std::env;

use reqwest::header::AUTHORIZATION;
use rustgram_server_util::db::StringEntity;
use rustgram_server_util::error::ServerErrorCodes;
use sentc_crypto::util::public::handle_server_response;
use sentc_crypto_common::user::UserDeviceRegisterInput;
use sentc_crypto_common::ServerOutput;
use sentc_crypto_light::sdk_common::user::DoneLoginServerReturn;
use server_api::util::api_res::ApiErrorCodes;
use server_dashboard_common::customer::{CustomerData, CustomerDoneLoginOutput, CustomerRegisterData, CustomerRegisterOutput, CustomerUpdateInput};
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{auth_header, get_captcha, get_url, login_customer};

mod test_fn;

pub struct CustomerState
{
	pub customer_id: String,
	pub customer_email: String,
	pub customer_pw: String,
	pub customer_data: Option<CustomerDoneLoginOutput>,
}

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerState>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_global()
{
	dotenv::from_filename("sentc.env").ok();

	CUSTOMER_TEST_STATE
		.get_or_init(|| {
			async move {
				//
				RwLock::new(CustomerState {
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
	let url = get_url("api/v1/customer/register".to_string());

	let wrong_email = "hello@localhost".to_string();

	let register_data = sentc_crypto_light::user::register(wrong_email.as_str(), "12345").unwrap();
	let register_data: UserDeviceRegisterInput = serde_json::from_str(register_data.as_str()).unwrap();

	let captcha_input = get_captcha().await;

	let input = CustomerRegisterData {
		customer_data: CustomerData {
			name: "abc".to_string(),
			first_name: "abc".to_string(),
			company: None,
		},
		email: wrong_email,
		register_data,
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

	let register_data = sentc_crypto_light::user::register(email.as_str(), "12345").unwrap();
	let register_data: UserDeviceRegisterInput = serde_json::from_str(register_data.as_str()).unwrap();

	let captcha_input = get_captcha().await;

	let input = CustomerRegisterData {
		customer_data: CustomerData {
			name: "abc".to_string(),
			first_name: "abc".to_string(),
			company: None,
		},
		email: email.to_string(),
		register_data,
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

	assert!(out.status);
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

	let url = get_url("api/v1/customer/prepare_login".to_owned());

	let prep_server_input = sentc_crypto_light::user::prepare_login_start(email).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = sentc_crypto_light::user::prepare_login(email, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/customer/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client.post(url).body(input).send().await.unwrap();

	let server_out = res.text().await.unwrap();

	match sentc_crypto_light::user::check_done_login(&server_out).unwrap() {
		DoneLoginServerReturn::Direct(d) => {
			let keys = sentc_crypto_light::user::done_login(&derived_master_key, auth_key, email.to_string(), d).unwrap();

			let url = get_url("api/v1/customer/verify_login".to_owned());
			let client = reqwest::Client::new();
			let res = client.post(url).body(keys.challenge).send().await.unwrap();

			let server_out = res.text().await.unwrap();

			let server_out: CustomerDoneLoginOutput = handle_server_response(&server_out).unwrap();

			customer.customer_data = Some(server_out);
		},
		DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for customer login")
		},
	}
}

#[tokio::test]
async fn test_13_aa_update_data()
{
	let mut customer = CUSTOMER_TEST_STATE.get().unwrap().write().await;
	let jwt = &customer.customer_data.as_ref().unwrap().verify.jwt;
	let email = &customer.customer_email;
	let pw = &customer.customer_pw;

	let input = CustomerData {
		name: "hello".to_string(),
		first_name: "my friend".to_string(),
		company: Some("abc".to_string()),
	};

	let url = get_url("api/v1/customer/data".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();

	//get the new values from log in

	let url = get_url("api/v1/customer/prepare_login".to_owned());

	let prep_server_input = sentc_crypto_light::user::prepare_login_start(email).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = sentc_crypto_light::user::prepare_login(email, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/customer/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client.post(url).body(input).send().await.unwrap();

	let server_out = res.text().await.unwrap();

	match sentc_crypto_light::user::check_done_login(&server_out).unwrap() {
		DoneLoginServerReturn::Direct(d) => {
			let keys = sentc_crypto_light::user::done_login(&derived_master_key, auth_key, email.to_string(), d).unwrap();

			let url = get_url("api/v1/customer/verify_login".to_owned());
			let client = reqwest::Client::new();
			let res = client.post(url).body(keys.challenge).send().await.unwrap();

			let server_out = res.text().await.unwrap();

			let out: CustomerDoneLoginOutput = handle_server_response(&server_out).unwrap();

			assert_eq!(out.email_data.name, "hello".to_string());

			customer.customer_data = Some(out);
		},
		DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for customer login")
		},
	}
}

#[tokio::test]
async fn test_13_update_customer()
{
	let mut customer = CUSTOMER_TEST_STATE.get().unwrap().write().await;

	let email = &customer.customer_email;
	let pw = &customer.customer_pw;
	let jwt = &customer.customer_data.as_ref().unwrap().verify.jwt;

	let new_email = "hello3@test.com".to_string();

	let update_data = CustomerUpdateInput {
		new_email: new_email.to_string(),
	};

	let url = get_url("api/v1/customer".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.body(serde_json::to_string(&update_data).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();

	//______________________________________________________________________________________________
	//it should not login with the old email

	let url = get_url("api/v1/customer/prepare_login".to_owned());

	let prep_server_input = sentc_crypto_light::user::prepare_login_start(email).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, _auth_key, _derived_master_key) = sentc_crypto_light::user::prepare_login(email, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/customer/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client.post(url).body(input).send().await.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<DoneLoginServerReturn>::from_string(body.as_str()).unwrap();

	assert!(!out.status);

	//______________________________________________________________________________________________
	//login with the right data

	let login_data = login_customer(new_email.as_str(), pw).await;

	customer.customer_data = Some(login_data);
	customer.customer_email = new_email;
}

//__________________________________________________________________________________________________

#[tokio::test]
async fn test_14_change_password()
{
	//
	let mut customer = CUSTOMER_TEST_STATE.get().unwrap().write().await;

	let email = &customer.customer_email;
	let pw = &customer.customer_pw;

	let new_pw = "987456";

	//______________________________________________________________________________________________
	//login again to get a fresh jwt

	let url = get_url("api/v1/customer/prepare_login".to_owned());

	let prep_server_input = sentc_crypto_light::user::prepare_login_start(email).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = sentc_crypto_light::user::prepare_login(email, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/customer/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client.post(url).body(input).send().await.unwrap();

	let server_out = res.text().await.unwrap();

	let (out, done_login_out) = match sentc_crypto_light::user::check_done_login(&server_out).unwrap() {
		DoneLoginServerReturn::Direct(d) => {
			let keys = sentc_crypto_light::user::done_login(&derived_master_key, auth_key, email.to_string(), d.clone()).unwrap();

			let url = get_url("api/v1/customer/verify_login".to_owned());
			let client = reqwest::Client::new();
			let res = client.post(url).body(keys.challenge).send().await.unwrap();

			let server_out = res.text().await.unwrap();

			let out: CustomerDoneLoginOutput = handle_server_response(&server_out).unwrap();

			(out, d)
		},
		DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for customer login")
		},
	};

	//use a new fresh jwt
	let jwt = out.verify.jwt.clone();

	//______________________________________________________________________________________________

	let input = sentc_crypto_light::user::change_password(pw, new_pw, body.as_str(), done_login_out).unwrap();

	let url = get_url("api/v1/customer/password".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(jwt.as_str()))
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();

	//______________________________________________________________________________________________
	//should not login with wrong password

	let url = get_url("api/v1/customer/prepare_login".to_owned());

	let prep_server_input = sentc_crypto::user::prepare_login_start(email).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//try login with the old pw
	let (input, _auth_key, _derived_master_key) = sentc_crypto_light::user::prepare_login(email, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/customer/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client.post(url).body(input).send().await.unwrap();

	let body_done_login = res.text().await.unwrap();

	let out = ServerOutput::<CustomerDoneLoginOutput>::from_string(body_done_login.as_str()).unwrap();

	assert!(!out.status);

	//______________________________________________________________________________________________
	//login with new password

	let login_data = login_customer(email.as_str(), new_pw).await;

	customer.customer_data = Some(login_data);
	customer.customer_pw = new_pw.to_string();
}

#[tokio::test]
async fn test_15_change_password_again_from_pw_change()
{
	//
	let mut customer = CUSTOMER_TEST_STATE.get().unwrap().write().await;

	let email = &customer.customer_email;
	let pw = &customer.customer_pw;

	let new_pw = "12345";

	//______________________________________________________________________________________________
	//login again to get a fresh jwt

	let url = get_url("api/v1/customer/prepare_login".to_owned());

	let prep_server_input = sentc_crypto::user::prepare_login_start(email).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = sentc_crypto::user::prepare_login(email, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/customer/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client.post(url).body(input).send().await.unwrap();

	let server_out = res.text().await.unwrap();

	let (out, done_login_out) = match sentc_crypto_light::user::check_done_login(&server_out).unwrap() {
		DoneLoginServerReturn::Direct(d) => {
			let keys = sentc_crypto_light::user::done_login(&derived_master_key, auth_key, email.to_string(), d.clone()).unwrap();

			let url = get_url("api/v1/customer/verify_login".to_owned());
			let client = reqwest::Client::new();
			let res = client.post(url).body(keys.challenge).send().await.unwrap();

			let server_out = res.text().await.unwrap();

			let out: CustomerDoneLoginOutput = handle_server_response(&server_out).unwrap();

			(out, d)
		},
		DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for customer login")
		},
	};

	//use a new fresh jwt
	let jwt = out.verify.jwt.clone();

	//______________________________________________________________________________________________

	let input = sentc_crypto_light::user::change_password(pw, new_pw, body.as_str(), done_login_out).unwrap();

	let url = get_url("api/v1/customer/password".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(jwt.as_str()))
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();

	//______________________________________________________________________________________________
	//should not login with wrong password

	let url = get_url("api/v1/customer/prepare_login".to_owned());

	let prep_server_input = sentc_crypto::user::prepare_login_start(email).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//try login with the old pw
	let (input, _auth_key, _derived_master_key) = sentc_crypto::user::prepare_login(email, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/customer/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client.post(url).body(input).send().await.unwrap();

	let body_done_login = res.text().await.unwrap();

	let out = ServerOutput::<CustomerDoneLoginOutput>::from_string(body_done_login.as_str()).unwrap();

	assert!(!out.status);

	//______________________________________________________________________________________________
	//login with new password

	let login_data = login_customer(email.as_str(), new_pw).await;

	customer.customer_data = Some(login_data);
	customer.customer_pw = new_pw.to_string();
}

//__________________________________________________________________________________________________

#[tokio::test]
async fn test_16_reset_customer_password()
{
	//
	let mut customer = CUSTOMER_TEST_STATE.get().unwrap().write().await;

	let email = &customer.customer_email;
	let id = customer.customer_id.to_string();
	let pw = &customer.customer_pw;

	let new_pw = "123456789";

	let captcha_input = get_captcha().await;

	let input = server_dashboard_common::customer::CustomerResetPasswordInput {
		email: email.to_string(),
		captcha_input,
	};

	let url = get_url("api/v1/customer/password_reset".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();

	//change the db path of sqlite
	dotenv::from_filename("sentc.env").ok();
	env::set_var("DB_PATH", env::var("DB_PATH_TEST").unwrap());

	//get the token -> in real app the token gets send by email.
	server_api::start().await;

	//language=SQL
	let sql = "SELECT email_token FROM sentc_customer WHERE id = ?";

	let token: Option<StringEntity> = rustgram_server_util::db::query_first(sql, rustgram_server_util::set_params!(id))
		.await
		.unwrap();
	let token = token.unwrap().0;

	let reset_password_data = sentc_crypto_light::user::register(email, new_pw).unwrap();
	let reset_password_data: UserDeviceRegisterInput = serde_json::from_str(&reset_password_data).unwrap();

	let input = server_dashboard_common::customer::CustomerDonePasswordResetInput {
		token,
		reset_password_data,
	};

	let url = get_url("api/v1/customer/password_reset_validation".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();

	//______________________________________________________________________________________________
	//should not login with old pw

	let url = get_url("api/v1/customer/prepare_login".to_owned());

	let prep_server_input = sentc_crypto_light::user::prepare_login_start(email).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//try login with the old pw
	let (input, _auth_key, _derived_master_key) = sentc_crypto_light::user::prepare_login(email, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/customer/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client.post(url).body(input).send().await.unwrap();

	let body_done_login = res.text().await.unwrap();

	let out = ServerOutput::<CustomerDoneLoginOutput>::from_string(body_done_login.as_str()).unwrap();

	assert!(!out.status);

	//______________________________________________________________________________________________
	//login with new password

	let login_data = login_customer(email.as_str(), new_pw).await;

	customer.customer_data = Some(login_data);
	customer.customer_pw = new_pw.to_string();
}

//__________________________________________________________________________________________________

#[tokio::test]
async fn test_30_delete_customer()
{
	let customer = CUSTOMER_TEST_STATE.get().unwrap().read().await;

	let jwt = &customer.customer_data.as_ref().unwrap().verify.jwt;

	let url = get_url("api/v1/customer".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();
}

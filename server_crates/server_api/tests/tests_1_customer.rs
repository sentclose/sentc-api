use std::env;

use reqwest::header::AUTHORIZATION;
use sentc_crypto_common::user::{
	ChangePasswordData,
	DoneLoginServerInput,
	DoneLoginServerKeysOutput,
	DoneLoginServerOutput,
	PrepareLoginSaltServerOutput,
	RegisterData,
};
use sentc_crypto_common::ServerOutput;
use server_api::util::api_res::ApiErrorCodes;
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
		register_data: register_data.device,
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
		register_data: register_data.device,
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
		register_data: register_data.device,
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

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();

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
async fn test_14_change_password()
{
	//
	let mut customer = CUSTOMER_TEST_STATE.get().unwrap().write().await;

	let email = &customer.customer_email;
	let pw = &customer.customer_pw;
	let public_token = customer.public_token.as_str();

	let new_pw = "987456";

	//______________________________________________________________________________________________
	//login again to get a fresh jwt

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
		.body(auth_key.to_string())
		.send()
		.await
		.unwrap();

	let body_done_login = res.text().await.unwrap();

	let out = ServerOutput::<CustomerDoneLoginOutput>::from_string(body_done_login.as_str()).unwrap();

	//use a new fresh jwt
	let jwt = out.result.unwrap().user_keys.jwt;

	//______________________________________________________________________________________________
	let pw_change_data = get_fake_pw_change_data(auth_key.as_str(), pw, new_pw);

	let url = get_url("api/v1/customer/password".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", public_token)
		.header(AUTHORIZATION, auth_header(jwt.as_str()))
		.body(pw_change_data)
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
		.header("x-sentc-app-token", public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//try login with the old pw
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

	let body_done_login = res.text().await.unwrap();

	let out = ServerOutput::<CustomerDoneLoginOutput>::from_string(body_done_login.as_str()).unwrap();

	assert_eq!(out.status, false);

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
	let public_token = customer.public_token.as_str();

	let new_pw = "12345";

	//______________________________________________________________________________________________
	//login again to get a fresh jwt

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
		.body(auth_key.to_string())
		.send()
		.await
		.unwrap();

	let body_done_login = res.text().await.unwrap();

	let out = ServerOutput::<CustomerDoneLoginOutput>::from_string(body_done_login.as_str()).unwrap();

	//use a new fresh jwt
	let jwt = out.result.unwrap().user_keys.jwt;

	//______________________________________________________________________________________________
	let pw_change_data = get_fake_pw_change_data(auth_key.as_str(), pw, new_pw);

	let url = get_url("api/v1/customer/password".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", public_token)
		.header(AUTHORIZATION, auth_header(jwt.as_str()))
		.body(pw_change_data)
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
		.header("x-sentc-app-token", public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//try login with the old pw
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

	let body_done_login = res.text().await.unwrap();

	let out = ServerOutput::<CustomerDoneLoginOutput>::from_string(body_done_login.as_str()).unwrap();

	assert_eq!(out.status, false);

	//______________________________________________________________________________________________
	//login with new password

	let login_data = login_customer(email.as_str(), new_pw).await;

	customer.customer_data = Some(login_data);
	customer.customer_pw = new_pw.to_string();
}

fn get_fake_login_data(old_pw: &str) -> (String, String)
{
	//use a fake master key to change the password,
	// just register the user again with fake data but with the old password to decrypt the fake data!
	let fake_key_data = sentc_crypto::user::register("abc", old_pw).unwrap();
	let fake_key_data = RegisterData::from_string(fake_key_data.as_str()).unwrap();

	//do the server prepare login again to get the salt (we need a salt to this fake register data)
	let salt_string = sentc_crypto::util::server::generate_salt_from_base64_to_string(
		fake_key_data.device.derived.client_random_value.as_str(),
		fake_key_data.device.derived.derived_alg.as_str(),
		"",
	)
	.unwrap();

	let prepare_login_user_data = PrepareLoginSaltServerOutput {
		salt_string,
		derived_encryption_key_alg: fake_key_data.device.derived.derived_alg.to_string(),
	};

	let device_keys = DoneLoginServerKeysOutput {
		encrypted_master_key: fake_key_data.device.master_key.encrypted_master_key,
		encrypted_private_key: fake_key_data.device.derived.encrypted_private_key,
		public_key_string: fake_key_data.device.derived.public_key,
		keypair_encrypt_alg: fake_key_data.device.derived.keypair_encrypt_alg,
		encrypted_sign_key: fake_key_data.device.derived.encrypted_sign_key,
		verify_key_string: fake_key_data.device.derived.verify_key,
		keypair_sign_alg: fake_key_data.device.derived.keypair_sign_alg,
		keypair_encrypt_id: "abc".to_string(),
		keypair_sign_id: "abc".to_string(),
		user_id: "abc".to_string(),
		device_id: "1234".to_string(),
		user_group_id: "1234".to_string(),
	};

	let done_login_user_data = DoneLoginServerOutput {
		device_keys,
		jwt: "abc".to_string(),
		refresh_token: "abc".to_string(),
		user_keys: vec![],
	};

	let prepare_login_user_data = ServerOutput {
		status: true,
		err_msg: None,
		err_code: None,
		result: Some(prepare_login_user_data),
	};
	let prepare_login_user_data = prepare_login_user_data.to_string().unwrap();

	let done_login_user_data = ServerOutput {
		status: true,
		err_msg: None,
		err_code: None,
		result: Some(done_login_user_data),
	};
	let done_login_user_data = done_login_user_data.to_string().unwrap();

	(prepare_login_user_data, done_login_user_data)
}

fn get_fake_pw_change_data(prepare_login_auth_key_input: &str, old_pw: &str, new_pw: &str) -> String
{
	//TODo do this in the dashboard client

	let (prepare_login_user_data, done_login_user_data) = get_fake_login_data(old_pw);

	let pw_change_data = sentc_crypto::user::change_password(
		old_pw,
		new_pw,
		prepare_login_user_data.as_str(),
		done_login_user_data.as_str(),
	)
	.unwrap();

	//set the old auth key from the prepare login (because we are using non registered fake data for pw change)
	let mut pw_change_data = ChangePasswordData::from_string(pw_change_data.as_str()).unwrap();
	//the auth key from prepare login returns the server input as string -> get here the auth key
	let auth_key = DoneLoginServerInput::from_string(prepare_login_auth_key_input).unwrap();

	pw_change_data.old_auth_key = auth_key.auth_key;
	let pw_change_data = pw_change_data.to_string().unwrap();

	pw_change_data
}

//__________________________________________________________________________________________________

#[tokio::test]
async fn test_16_reset_customer_password()
{
	//
	let mut customer = CUSTOMER_TEST_STATE.get().unwrap().write().await;

	let email = &customer.customer_email;
	let public_token = customer.public_token.as_str();
	let id = customer.customer_id.to_string();
	let pw = &customer.customer_pw;

	let new_pw = "123456789";

	let input = server_api_common::customer::CustomerResetPasswordInput {
		email: email.to_string(),
	};

	let url = get_url("api/v1/customer/password_reset".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", public_token)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();

	//change the db path of sqlite
	dotenv::dotenv().ok();
	env::set_var("DB_PATH", env::var("DB_PATH_TEST").unwrap());

	//get the token -> in real app the token gets send by email.
	server_api::start().await;

	//language=SQL
	let sql = "SELECT email_token FROM sentc_customer WHERE id = ?";

	let token: Option<CustomerEmailToken> = server_core::db::query_first(sql, server_core::set_params!(id))
		.await
		.unwrap();
	let token = token.unwrap().email_token;

	//make a fake register and login to get fake decrypted private keys, then do the pw reset like normal user
	//use a rand pw to generate the fake keys
	let (prepare_login_user_data, done_login_user_data) = get_fake_login_data("abc");

	//use the same fake pw here
	let (_auth_key, derived_master_key) = sentc_crypto::user::prepare_login(email, "abc", prepare_login_user_data.as_str()).unwrap();

	let user_key_data = sentc_crypto::user::done_login(&derived_master_key, done_login_user_data.as_str()).unwrap();

	let pw_reset_out = sentc_crypto::user::reset_password(
		new_pw,
		&user_key_data.device_keys.private_key,
		&user_key_data.device_keys.sign_key,
	)
	.unwrap();
	let reset_password_data = sentc_crypto_common::user::ResetPasswordData::from_string(pw_reset_out.as_str()).unwrap();

	let input = server_api_common::customer::CustomerDonePasswordResetInput {
		token,
		reset_password_data,
	};

	let url = get_url("api/v1/customer/password_reset_validation".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", public_token)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();

	//______________________________________________________________________________________________
	//should not login with old pw

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

	//try login with the old pw
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

	let body_done_login = res.text().await.unwrap();

	let out = ServerOutput::<CustomerDoneLoginOutput>::from_string(body_done_login.as_str()).unwrap();

	assert_eq!(out.status, false);

	//______________________________________________________________________________________________
	//login with new password

	let login_data = login_customer(email.as_str(), new_pw).await;

	customer.customer_data = Some(login_data);
	customer.customer_pw = new_pw.to_string();
}

pub(crate) struct CustomerEmailToken
{
	pub email_token: String,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for CustomerEmailToken
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			email_token: server_core::take_or_err!(row, 0, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for CustomerEmailToken
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			email_token: server_core::take_or_err!(row, 0),
		})
	}
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

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();
}

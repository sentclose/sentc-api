//This is about the user light

use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use rustgram_server_util::error::ServerErrorCodes;
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::user::{
	DoneLoginLightServerOutput,
	DoneLoginServerOutput,
	MasterKey,
	RegisterServerOutput,
	UserDeviceList,
	UserDeviceRegisterOutput,
	UserIdentifierAvailableServerInput,
	UserIdentifierAvailableServerOutput,
	UserInitServerOutput,
	UserUpdateServerInput,
};
use sentc_crypto_common::{ServerOutput, UserId};
use sentc_crypto_light::error::SdkLightError;
use sentc_crypto_light::sdk_utils::error::SdkUtilError;
use sentc_crypto_light::sdk_utils::{handle_general_server_response, handle_server_response};
use sentc_crypto_light::UserDataInt;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use server_api::util::api_res::ApiErrorCodes;
use server_api_common::app::AppRegisterOutput;
use server_api_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
	add_app_jwt_keys,
	auth_header,
	create_app,
	create_test_customer,
	customer_delete,
	delete_app,
	delete_app_jwt_key,
	get_server_error_from_normal_res,
	get_url,
	login_user_light,
};

mod test_fn;

pub struct UserState
{
	pub username: String,
	pub pw: String,
	pub user_id: UserId,
	pub user_data: Option<UserDataInt>,
	pub user_data_1: Option<UserDataInt>, //for the 2nd device
	pub app_data: AppRegisterOutput,
	pub customer_data: CustomerDoneLoginOutput,
}

static USER_TEST_STATE: OnceCell<RwLock<UserState>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_global_test()
{
	dotenv::from_filename("sentc.env").ok();

	let (_, customer_data) = create_test_customer("hello@test3.com", "12345").await;

	let customer_jwt = &customer_data.verify.jwt;

	//create here an app
	let app_data = create_app(customer_jwt).await;

	//this fn must be execute first!
	USER_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(UserState {
					username: "admin_test".to_string(),
					pw: "12345".to_string(),
					user_id: "".to_string(),
					user_data: None,
					user_data_1: None,
					app_data,
					customer_data,
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_11_user_register()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;

	let username = &user.username;
	let pw = &user.pw;

	let url = get_url("api/v1/register_light".to_owned());

	let input = sentc_crypto_light::user::register(username, pw).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	//check it here (not like the client) to see if the server respond correctly
	let register_out: RegisterServerOutput = handle_server_response(&body).unwrap();
	assert_eq!(register_out.device_identifier, username.to_string());

	//get the user id like the client
	let user_id = sentc_crypto_light::user::done_register(body.as_str()).unwrap();

	assert_ne!(user_id, "".to_owned());

	//save the user id
	user.user_id = user_id;
}

#[tokio::test]
async fn test_12_user_check_after_register()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let username = &user.username;

	//test if user exists
	let input = UserIdentifierAvailableServerInput {
		user_identifier: username.to_string(),
	};

	let url = get_url("api/v1/exists".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let exists: UserIdentifierAvailableServerOutput = handle_server_response(&body).unwrap();

	assert_eq!(exists.user_identifier, username.to_string());
	assert!(!exists.available);
}

#[tokio::test]
async fn test_13_user_register_failed_username_exists()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let username = &user.username;
	let pw = &user.pw;

	let url = get_url("api/v1/register_light".to_owned());

	let input = sentc_crypto_light::user::register(username, pw).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::BAD_REQUEST);

	let body = res.text().await.unwrap();
	let error = ServerOutput::<RegisterServerOutput>::from_string(body.as_str()).unwrap();

	assert!(!error.status);
	assert!(error.result.is_none());
	assert_eq!(error.err_code.unwrap(), ApiErrorCodes::UserExists.get_int_code());

	//check err in sdk
	match sentc_crypto_light::user::done_register(body.as_str()) {
		Ok(_v) => {
			panic!("this should not be Ok")
		},
		Err(e) => {
			match e {
				SdkLightError::Util(SdkUtilError::ServerErr(s, m)) => {
					//this should be the right err
					//this are the same err as the backend
					assert_eq!(error.err_code.unwrap(), s);
					assert_eq!(error.err_msg.unwrap(), m);
				},
				_ => panic!("this should not be the right error code"),
			}
		},
	}
}

#[tokio::test]
async fn test_14_login()
{
	//make the pre req to the sever with the username
	let url = get_url("api/v1/prepare_login".to_owned());

	let mut user = USER_TEST_STATE.get().unwrap().write().await;
	let username = &user.username;
	let pw = &user.pw;

	let prep_server_input = sentc_crypto_light::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = sentc_crypto_light::user::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let server_out = res.text().await.unwrap();

	let keys = match sentc_crypto_light::user::check_done_login(&server_out).unwrap() {
		sentc_crypto_light::sdk_common::user::DoneLoginServerReturn::Direct(d) => {
			sentc_crypto::user::done_login(&derived_master_key, auth_key, username.to_string(), d).unwrap()
		},
		sentc_crypto_light::sdk_common::user::DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for user login test 2")
		},
	};

	let url = get_url("api/v1/verify_login_light".to_owned());
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(keys.challenge)
		.send()
		.await
		.unwrap();
	let server_out = res.text().await.unwrap();

	let keys = sentc_crypto_light::user::verify_login(&server_out, keys.user_id, keys.device_id, keys.device_keys).unwrap();

	user.user_data = Some(keys);
}

#[tokio::test]
async fn test_15_login_with_wrong_password()
{
	//make the pre req to the sever with the username
	let url = get_url("api/v1/prepare_login".to_owned());

	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let username = &user.username;
	let pw = "wrong_password"; //the wording pw

	let prep_server_input = sentc_crypto_light::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, _auth_key, _derived_master_key) = sentc_crypto_light::user::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let login_output = ServerOutput::<DoneLoginServerOutput>::from_string(body.as_str()).unwrap();

	assert!(!login_output.status);
	assert!(login_output.result.is_none());
	assert_eq!(login_output.err_code.unwrap(), ApiErrorCodes::Login.get_int_code());
}

#[tokio::test]
async fn test_16_user_delete_with_wrong_jwt()
{
	//test user action with old jwt (when the jwt keys was deleted)

	//create new jwt keys for the app
	let mut user = USER_TEST_STATE.get().unwrap().write().await;

	let old_jwt = &user.user_data.as_ref().unwrap().jwt;
	let old_jwt_data = &user.app_data.jwt_data;

	let customer_jwt = &user.customer_data.verify.jwt;

	let new_keys = add_app_jwt_keys(customer_jwt, user.app_data.app_id.as_str()).await;

	//delete the old jwt
	delete_app_jwt_key(
		customer_jwt,
		old_jwt_data.app_id.as_str(),
		old_jwt_data.jwt_id.as_str(),
	)
	.await;

	let url = get_url("api/v1/user".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(old_jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let delete_output = ServerOutput::<ServerSuccessOutput>::from_string(body.as_str()).unwrap();

	assert!(!delete_output.status);

	//login new in to get the new jwt with the new keys
	let username = &user.username;
	let pw = &user.pw;

	let public_token = &user.app_data.public_token;

	let login = login_user_light(public_token, username, pw).await;

	user.app_data.jwt_data = new_keys; //save the new jwt for other tests
	user.user_data = Some(login);
}

#[tokio::test]
async fn test_17_change_user_pw()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;
	let username = &user.username;
	let pw = &user.pw;
	let new_pw = "54321";

	let public_token = &user.app_data.public_token;

	//______________________________________________________________________________________________
	//1. do prep login to get the auth key

	let prep_server_input = sentc_crypto_light::user::prepare_login_start(username).unwrap();

	let url = get_url("api/v1/prepare_login".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//to still prep login from sdk to get the auth key for done login
	let (input, auth_key, derived_master_key) = sentc_crypto_light::user::prepare_login(username, pw, body.as_str()).unwrap();

	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let done_body = res.text().await.unwrap();

	let (keys, done_body) = match sentc_crypto_light::user::check_done_login(&done_body).unwrap() {
		sentc_crypto_light::sdk_common::user::DoneLoginServerReturn::Direct(d) => {
			(
				sentc_crypto::user::done_login(&derived_master_key, auth_key, username.to_string(), d.clone()).unwrap(),
				d,
			)
		},
		sentc_crypto_light::sdk_common::user::DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for user login test 2")
		},
	};

	//2. login again to get a fresh jwt

	let url = get_url("api/v1/verify_login_light".to_owned());
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(keys.challenge)
		.send()
		.await
		.unwrap();
	let server_out = res.text().await.unwrap();

	let keys = sentc_crypto_light::user::verify_login(&server_out, keys.user_id, keys.device_id, keys.device_keys).unwrap();

	let jwt = keys.jwt;

	//______________________________________________________________________________________________
	//use a fresh jwt here

	let input = sentc_crypto_light::user::change_password(pw, new_pw, body.as_str(), done_body).unwrap();

	let url = get_url("api/v1/user/update_pw".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", public_token)
		.header(AUTHORIZATION, auth_header(jwt.as_str()))
		.body(input)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	handle_general_server_response(&body).unwrap();

	//______________________________________________________________________________________________
	//try to login with old pw
	let url = get_url("api/v1/prepare_login".to_owned());

	let prep_server_input = sentc_crypto_light::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, _auth_key, _derived_master_key) = sentc_crypto_light::user::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let login_output = ServerOutput::<DoneLoginServerOutput>::from_string(body.as_str()).unwrap();

	assert!(!login_output.status);
	assert!(login_output.result.is_none());
	assert_eq!(login_output.err_code.unwrap(), ApiErrorCodes::Login.get_int_code());

	//______________________________________________________________________________________________
	//login with new password
	let login = login_user_light(public_token, username, new_pw).await;

	//the the new key data
	user.user_data = Some(login);
	user.pw = new_pw.to_string();
}

#[tokio::test]
async fn test_18_reset_password()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;
	let username = &user.username;
	let old_pw = &user.pw;
	let new_pw = "a_new_password";
	let secret_token = &user.app_data.secret_token; //pw light reset is server side only

	let reset_password_data = sentc_crypto_light::user::register(username, new_pw).unwrap();

	let url = get_url("api/v1/user/reset_pw_light".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", secret_token)
		.body(reset_password_data)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();

	//______________________________________________________________________________________________
	//test login with the old pw

	let url = get_url("api/v1/prepare_login".to_owned());

	let prep_server_input = sentc_crypto_light::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, _auth_key, _derived_master_key) = sentc_crypto_light::user::prepare_login(username, old_pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let login_output = ServerOutput::<DoneLoginServerOutput>::from_string(body.as_str()).unwrap();

	assert!(!login_output.status);
	assert!(login_output.result.is_none());
	assert_eq!(login_output.err_code.unwrap(), ApiErrorCodes::Login.get_int_code());

	//______________________________________________________________________________________________
	//test login with new pw
	let login = login_user_light(secret_token, username, new_pw).await;

	//the the new key data
	user.user_data = Some(login);
	user.pw = new_pw.to_string();
}

#[tokio::test]
async fn test_22_refresh_jwt()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt;
	let username = &user.username;

	let input = sentc_crypto_light::user::prepare_refresh_jwt(user.user_data.as_ref().unwrap().refresh_token.clone()).unwrap();

	let url = get_url("api/v1/refresh".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.header(AUTHORIZATION, auth_header(jwt))
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<DoneLoginLightServerOutput>::from_string(body.as_str()).unwrap();

	assert!(out.status);

	let out = out.result.unwrap();

	let non_fresh_jwt = out.jwt;

	//don't need to change jwt in user data because the old one is still valid

	//______________________________________________________________________________________________

	//check new jwt -> the error here comes from the wrong update but not from a wrong jwt
	let input = UserUpdateServerInput {
		user_identifier: username.to_string(),
	};

	let url = get_url("api/v1/user".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.body(input.to_string().unwrap())
		.header("x-sentc-app-token", &user.app_data.public_token)
		.header(AUTHORIZATION, auth_header(non_fresh_jwt.as_str()))
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::BAD_REQUEST);

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, 101);

	//______________________________________________________________________________________________
	//it should not delete the user because a fresh jwt is needed here.
	let user = &USER_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/user".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(non_fresh_jwt.as_str()))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out = ServerOutput::<ServerSuccessOutput>::from_string(body.as_str()).unwrap();

	assert!(!out.status);
	assert_eq!(out.err_code.unwrap(), ApiErrorCodes::WrongJwtAction.get_int_code());
}

#[tokio::test]
async fn test_23_user_normal_init()
{
	//no group invite here at this point

	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt;

	let url = get_url("api/v1/init".to_owned());

	let input = sentc_crypto_light::user::prepare_refresh_jwt(user.user_data.as_ref().unwrap().refresh_token.clone()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<UserInitServerOutput>::from_string(body.as_str()).unwrap();

	assert!(out.status);

	let out = out.result.unwrap();

	//don't save the jwt because we need a fresh jwt
	assert_eq!(out.invites.len(), 0);
}

#[tokio::test]
async fn test_24_user_add_device()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt;

	let input = sentc_crypto_light::user::register("device_1", "12345").unwrap();

	let url = get_url("api/v1/user/prepare_register_device".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//normal check if the res was ok
	let _out: UserDeviceRegisterOutput = handle_server_response(body.as_str()).unwrap();

	//check in the sdk if the res was ok
	sentc_crypto_light::user::done_register_device_start(body.as_str()).unwrap();

	//now transfer the output to the main device to add it
	let input = sentc_crypto_light::user::prepare_register_device(body.as_str()).unwrap();

	let url = get_url("api/v1/user/done_register_device_light".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//login the new device
	let url = get_url("api/v1/prepare_login".to_owned());

	let prep_server_input = sentc_crypto_light::user::prepare_login_start("device_1").unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = sentc_crypto_light::user::prepare_login("device_1", "12345", body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let server_out = res.text().await.unwrap();

	let keys = match sentc_crypto_light::user::check_done_login(&server_out).unwrap() {
		sentc_crypto_light::sdk_common::user::DoneLoginServerReturn::Direct(d) => {
			sentc_crypto::user::done_login(&derived_master_key, auth_key, "device_1".to_string(), d).unwrap()
		},
		sentc_crypto_light::sdk_common::user::DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for user login test 2")
		},
	};

	let url = get_url("api/v1/verify_login_light".to_owned());
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(keys.challenge)
		.send()
		.await
		.unwrap();
	let server_out = res.text().await.unwrap();

	let keys = sentc_crypto_light::user::verify_login(&server_out, keys.user_id, keys.device_id, keys.device_keys).unwrap();

	assert_ne!(
		&keys.device_keys.private_key.key_id,
		&user
			.user_data
			.as_ref()
			.unwrap()
			.device_keys
			.private_key
			.key_id
	);

	user.user_data_1 = Some(keys);
}

#[tokio::test]
async fn test_25_get_all_devices()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt; //use the jwt from the main device

	let url = get_url("api/v1/user/device/0/none".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<UserDeviceList> = handle_server_response(body.as_str()).unwrap();

	assert_eq!(list.len(), 2);
	assert_eq!(list[0].device_id, user.user_data.as_ref().unwrap().device_id);
	assert_eq!(list[1].device_id, user.user_data_1.as_ref().unwrap().device_id);

	//get device from page 2

	let last_item = &list[0];

	let url = get_url("api/v1/user/device/".to_owned() + last_item.time.to_string().as_str() + "/" + last_item.device_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<UserDeviceList> = handle_server_response(body.as_str()).unwrap();

	assert_eq!(list.len(), 1);
	assert_eq!(list[0].device_id, user.user_data_1.as_ref().unwrap().device_id);
}

#[tokio::test]
async fn test_28_delete_device()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt; //use the jwt from the main device

	let user_data = user.user_data_1.as_ref().unwrap();

	let url = get_url("api/v1/user/device/".to_owned() + user_data.device_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();

	//should not login with the deleted device
	let url = get_url("api/v1/prepare_login".to_owned());

	let prep_server_input = sentc_crypto_light::user::prepare_login_start("device_1").unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, _auth_key, _derived_master_key) = sentc_crypto_light::user::prepare_login("device_1", "12345", body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	match handle_server_response::<sentc_crypto_light::sdk_common::user::DoneLoginServerReturn>(&body) {
		Ok(_) => panic!("should be error"),
		Err(e) => {
			match e {
				SdkUtilError::ServerErr(s, _) => {
					assert_eq!(s, 100)
				},
				_ => panic!("should be server error"),
			}
		},
	}
}

#[tokio::test]
async fn test_29_not_delete_the_last_device()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt; //use the jwt from the main device

	let user_data = user.user_data.as_ref().unwrap();

	let url = get_url("api/v1/user/device/".to_owned() + user_data.device_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, 115);
}

//do user tests before this one!

#[tokio::test]
async fn test_40_user_delete()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt;

	let url = get_url("api/v1/user".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct WrongRegisterData
{
	pub master_key: MasterKey,
}

impl WrongRegisterData
{
	pub fn from_string(v: &str) -> serde_json::Result<Self>
	{
		from_str::<Self>(v)
	}

	pub fn to_string(&self) -> serde_json::Result<String>
	{
		to_string(self)
	}
}

#[tokio::test]
async fn test_41_not_register_user_with_wrong_input()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/register_light".to_owned());

	let input = WrongRegisterData {
		master_key: MasterKey {
			master_key_alg: "123".to_string(),
			encrypted_master_key: "321".to_string(),
			encrypted_master_key_alg: "11".to_string(),
		},
	};

	let str = input.to_string().unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.body(str)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

	let body = res.text().await.unwrap();
	let error = ServerOutput::<RegisterServerOutput>::from_string(body.as_str()).unwrap();

	assert!(!error.status);
	assert!(error.result.is_none());
	assert_eq!(error.err_code.unwrap(), ApiErrorCodes::JsonParse.get_int_code());

	//check err in sdk
	match sentc_crypto_light::user::done_register(body.as_str()) {
		Ok(_v) => {
			panic!("this should not be Ok")
		},
		Err(e) => {
			match e {
				SdkLightError::Util(SdkUtilError::ServerErr(s, m)) => {
					//this should be the right err
					//this are the same err as the backend
					assert_eq!(error.err_code.unwrap(), s);
					assert_eq!(error.err_msg.unwrap(), m);
				},
				_ => panic!("this should not be the right error code"),
			}
		},
	}
}

#[tokio::test]
async fn zzz_clean_up()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let customer_jwt = &user.customer_data.verify.jwt;

	delete_app(customer_jwt, user.app_data.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

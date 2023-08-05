use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use rustgram_server_util::error::ServerErrorCodes;
use sentc_crypto::entities::user::UserDataInt;
use sentc_crypto::sdk_common::group::{GroupAcceptJoinReqServerOutput, KeyRotationInput};
use sentc_crypto::sdk_common::user::UserDeviceRegisterOutput;
use sentc_crypto::sdk_utils::error::SdkUtilError;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::SdkError;
use sentc_crypto_common::group::{GroupKeyServerOutput, KeyRotationStartServerOutput};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::user::{
	DoneLoginLightServerOutput,
	DoneLoginServerOutput,
	MasterKey,
	RegisterServerOutput,
	UserDeviceList,
	UserIdentifierAvailableServerInput,
	UserIdentifierAvailableServerOutput,
	UserInitServerOutput,
	UserJwtInfo,
	UserPublicKeyDataServerOutput,
	UserUpdateServerInput,
	UserVerifyKeyDataServerOutput,
	VerifyLoginOutput,
};
use sentc_crypto_common::{ServerOutput, UserId};
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
	delete_user,
	get_server_error_from_normal_res,
	get_url,
	login_user,
	register_user,
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
async fn test_10_user_exists()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let username = &user.username;

	//test if user exists
	let input = UserIdentifierAvailableServerInput {
		user_identifier: username.to_owned(),
	};

	let url = get_url("api/v1/exists".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(input.to_string().unwrap())
		.header("x-sentc-app-token", &user.app_data.public_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let exists = ServerOutput::<UserIdentifierAvailableServerOutput>::from_string(body.as_str()).unwrap();

	assert!(exists.status);
	assert_eq!(exists.err_code, None);

	let exists = match exists.result {
		Some(v) => v,
		None => panic!("exists is not here"),
	};

	assert_eq!(exists.user_identifier, username.to_string());
	assert!(exists.available);
}

#[tokio::test]
async fn test_11_user_register()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;

	let username = &user.username;
	let pw = &user.pw;

	let url = get_url("api/v1/register".to_owned());

	let input = sentc_crypto::user::register(username, pw).unwrap();

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
	let register_out = ServerOutput::<RegisterServerOutput>::from_string(body.as_str()).unwrap();

	assert!(register_out.status);
	assert_eq!(register_out.err_code, None);

	let register_out = register_out.result.unwrap();
	assert_eq!(register_out.device_identifier, username.to_string());

	//get the user id like the client
	let user_id = sentc_crypto::user::done_register(body.as_str()).unwrap();

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

	let exists = ServerOutput::<UserIdentifierAvailableServerOutput>::from_string(body.as_str()).unwrap();

	assert!(exists.status);
	assert_eq!(exists.err_code, None);

	let exists = exists.result.unwrap();

	assert_eq!(exists.user_identifier, username.to_string());
	assert!(!exists.available);
}

#[tokio::test]
async fn test_13_user_register_failed_username_exists()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let username = &user.username;
	let pw = &user.pw;

	let url = get_url("api/v1/register".to_owned());

	let input = sentc_crypto::user::register(username, pw).unwrap();

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
	match sentc_crypto::user::done_register(body.as_str()) {
		Ok(_v) => {
			panic!("this should not be Ok")
		},
		Err(e) => {
			match e {
				SdkError::Util(SdkUtilError::ServerErr(s, m)) => {
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

	let prep_server_input = sentc_crypto::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = sentc_crypto::user::prepare_login(username, pw, body.as_str()).unwrap();

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

	let keys = match sentc_crypto::user::check_done_login(&server_out).unwrap() {
		sentc_crypto::sdk_common::user::DoneLoginServerReturn::Direct(d) => {
			sentc_crypto::user::done_login(&derived_master_key, auth_key, username.to_string(), d).unwrap()
		},
		sentc_crypto::sdk_common::user::DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for user login test 1")
		},
	};

	let url = get_url("api/v1/verify_login".to_owned());
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(keys.challenge)
		.send()
		.await
		.unwrap();
	let server_out = res.text().await.unwrap();

	let keys = sentc_crypto::user::verify_login(&server_out, keys.user_id, keys.device_id, keys.device_keys).unwrap();

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

	let prep_server_input = sentc_crypto::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, _auth_key, _derived_master_key) = sentc_crypto::user::prepare_login(username, pw, body.as_str()).unwrap();

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
async fn test_15_login_with_wrong_challenge()
{
	//make the pre req to the sever with the username
	let url = get_url("api/v1/prepare_login".to_owned());

	let user = USER_TEST_STATE.get().unwrap().read().await;
	let username = &user.username;
	let pw = &user.pw;

	let prep_server_input = sentc_crypto::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = sentc_crypto::user::prepare_login(username, pw, body.as_str()).unwrap();

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

	let keys = match sentc_crypto::user::check_done_login(&server_out).unwrap() {
		sentc_crypto::sdk_common::user::DoneLoginServerReturn::Direct(d) => {
			sentc_crypto::user::done_login(&derived_master_key, auth_key, username.to_string(), d).unwrap()
		},
		sentc_crypto::sdk_common::user::DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for user login test 1")
		},
	};

	//wrong challenge
	let challenge = keys.challenge + "abc";

	let url = get_url("api/v1/verify_login".to_owned());
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(challenge)
		.send()
		.await
		.unwrap();

	let server_out = res.text().await.unwrap();
	let login_output = ServerOutput::<VerifyLoginOutput>::from_string(&server_out).unwrap();
	assert!(!login_output.status);
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

	let login = login_user(public_token, username, pw).await;

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

	let prep_server_input = sentc_crypto::user::prepare_login_start(username).unwrap();

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
	let (input, auth_key, derived_master_key) = sentc_crypto::user::prepare_login(username, pw, body.as_str()).unwrap();

	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let done_login_out = res.text().await.unwrap();

	let (keys, done_login_out) = match sentc_crypto::user::check_done_login(&done_login_out).unwrap() {
		sentc_crypto::sdk_common::user::DoneLoginServerReturn::Direct(d) => {
			(
				sentc_crypto::user::done_login(&derived_master_key, auth_key, username.to_string(), d.clone()).unwrap(),
				d,
			)
		},
		sentc_crypto::sdk_common::user::DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for user login test 1")
		},
	};

	let url = get_url("api/v1/verify_login".to_owned());
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
		.body(keys.challenge)
		.send()
		.await
		.unwrap();
	let server_out = res.text().await.unwrap();

	let keys = sentc_crypto::user::verify_login(&server_out, keys.user_id, keys.device_id, keys.device_keys).unwrap();

	//2. login again to get a fresh jwt
	let jwt = keys.jwt;

	//______________________________________________________________________________________________
	//use a fresh jwt here

	let input = sentc_crypto::user::change_password(pw, new_pw, body.as_str(), done_login_out).unwrap();

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
	let out = ServerOutput::<ServerSuccessOutput>::from_string(body.as_str()).unwrap();

	assert!(out.status);
	assert_eq!(out.err_code, None);

	//______________________________________________________________________________________________
	//try to login with old pw
	let url = get_url("api/v1/prepare_login".to_owned());

	let prep_server_input = sentc_crypto::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, _auth_key, _derived_master_key) = sentc_crypto::user::prepare_login(username, pw, body.as_str()).unwrap();

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
	let login = login_user(public_token, username, new_pw).await;

	//the the new key data
	user.user_data = Some(login);
	user.pw = new_pw.to_string();
}

#[tokio::test]
async fn test_18_reset_password()
{
	//no prep and done login req like change password
	//but we need the decrypted private and sign key!

	let mut user = USER_TEST_STATE.get().unwrap().write().await;
	let username = &user.username;
	let old_pw = &user.pw;
	let new_pw = "a_new_password";
	let key_data = user.user_data.as_ref().unwrap();
	let public_token = &user.app_data.public_token;

	//use device keys for pw reset
	let input = sentc_crypto::user::reset_password(
		new_pw,
		&key_data.device_keys.private_key,
		&key_data.device_keys.sign_key,
	)
	.unwrap();

	let url = get_url("api/v1/user/reset_pw".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", public_token)
		.header(AUTHORIZATION, auth_header(key_data.jwt.as_str()))
		.body(input)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();

	//______________________________________________________________________________________________
	//test login with the old pw

	let url = get_url("api/v1/prepare_login".to_owned());

	let prep_server_input = sentc_crypto::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, _auth_key, _derived_master_key) = sentc_crypto::user::prepare_login(username, old_pw, body.as_str()).unwrap();

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
	let login = login_user(public_token, username, new_pw).await;

	//the the new key data
	user.user_data = Some(login);
	user.pw = new_pw.to_string();
}

#[tokio::test]
async fn test_19_user_update()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt;

	let new_username = "bla".to_string();

	let input = UserUpdateServerInput {
		user_identifier: new_username.to_string(),
	};

	let url = get_url("api/v1/user".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.body(input.to_string().unwrap())
		.header("x-sentc-app-token", &user.app_data.public_token)
		.header(AUTHORIZATION, auth_header(jwt))
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();

	user.username = new_username;
}

#[tokio::test]
async fn test_20_user_not_update_when_identifier_exists()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt;

	let new_username = "bla".to_string();

	let input = UserUpdateServerInput {
		user_identifier: new_username.to_string(),
	};

	let url = get_url("api/v1/user".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.body(input.to_string().unwrap())
		.header("x-sentc-app-token", &user.app_data.public_token)
		.header(AUTHORIZATION, auth_header(jwt))
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::BAD_REQUEST);

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, 101);
}

#[tokio::test]
async fn test_21_get_user_public_data()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;

	//get user public key
	let url = get_url("api/v1/user/".to_owned() + user.user_id.as_str() + "/public_key");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<UserPublicKeyDataServerOutput>::from_string(body.as_str()).unwrap();

	assert!(out.status);

	let out = out.result.unwrap();

	assert_eq!(
		out.public_key_id,
		user.user_data.as_ref().unwrap().user_keys[0]
			.public_key
			.key_id
			.to_string()
	);

	//convert the user public key
	let public_key = sentc_crypto::util::public::import_public_key_from_string_into_format(&body).unwrap();

	assert_ne!(public_key.public_key_sig, None);
	assert_ne!(public_key.public_key_sig_key_id, None);

	//get user verify key by id
	let url = get_url("api/v1/user/".to_owned() + user.user_id.as_str() + "/verify_key/" + out.public_key_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<UserVerifyKeyDataServerOutput>::from_string(body.as_str()).unwrap();

	assert!(out.status);

	let out = out.result.unwrap();

	assert_eq!(
		out.verify_key_id,
		user.user_data.as_ref().unwrap().user_keys[0]
			.verify_key
			.key_id
			.to_string()
	);

	//convert the user verify key
	let verify_key = sentc_crypto::util::public::import_verify_key_from_string_into_format(&body).unwrap();

	//verify the public key
	let verify = sentc_crypto::user::verify_user_public_key(&verify_key, &public_key).unwrap();

	assert!(verify);
}

#[tokio::test]
async fn test_21_z_get_user_jwt_info()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt;

	let url = get_url("api/v1/user/jwt".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.header(AUTHORIZATION, auth_header(jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: UserJwtInfo = handle_server_response(&body).unwrap();

	assert_eq!(out.id, user.user_data.as_ref().unwrap().user_id);
}

#[tokio::test]
async fn test_22_refresh_jwt()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt;

	let input = sentc_crypto::user::prepare_refresh_jwt(user.user_data.as_ref().unwrap().refresh_token.clone()).unwrap();

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
	let new_username = "bla".to_string();

	let input = UserUpdateServerInput {
		user_identifier: new_username.to_string(),
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

	let input = sentc_crypto::user::prepare_refresh_jwt(user.user_data.as_ref().unwrap().refresh_token.clone()).unwrap();

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

	let input = sentc_crypto::user::prepare_register_device_start("device_1", "12345").unwrap();

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
	sentc_crypto::user::done_register_device_start(body.as_str()).unwrap();

	//get the user group keys
	let group_keys = &user.user_data.as_ref().unwrap().user_keys;

	let mut group_keys_ref = vec![];

	for decrypted_group_key in group_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	//now transfer the output to the main device to add it
	let (input, _) = sentc_crypto::user::prepare_register_device(body.as_str(), &group_keys_ref, false).unwrap();

	let url = get_url("api/v1/user/done_register_device".to_owned());

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

	let _out: GroupAcceptJoinReqServerOutput = handle_server_response(body.as_str()).unwrap();

	//no key session yet

	//login the new device
	let url = get_url("api/v1/prepare_login".to_owned());

	let prep_server_input = sentc_crypto::user::prepare_login_start("device_1").unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = sentc_crypto::user::prepare_login("device_1", "12345", body.as_str()).unwrap();

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

	let done_login_out = res.text().await.unwrap();

	let keys = match sentc_crypto::user::check_done_login(&done_login_out).unwrap() {
		sentc_crypto::sdk_common::user::DoneLoginServerReturn::Direct(d) => {
			sentc_crypto::user::done_login(&derived_master_key, auth_key, "device_1".to_string(), d).unwrap()
		},
		sentc_crypto::sdk_common::user::DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for user login test 1")
		},
	};

	let url = get_url("api/v1/verify_login".to_owned());
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(keys.challenge)
		.send()
		.await
		.unwrap();
	let server_out = res.text().await.unwrap();

	let keys = sentc_crypto::user::verify_login(&server_out, keys.user_id, keys.device_id, keys.device_keys).unwrap();

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
async fn test_26_user_group_key_rotation()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt; //use the jwt from the main device

	let pre_group_key = &user.user_data.as_ref().unwrap().user_keys[0].group_key;
	let device_invoker_public_key = &user.user_data.as_ref().unwrap().device_keys.public_key;

	let input = sentc_crypto::group::key_rotation(
		pre_group_key,
		device_invoker_public_key,
		true,
		None,
		"test".to_string(),
	)
	.unwrap();

	let url = get_url("api/v1/user/user_keys/rotation".to_string());
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let key_out: KeyRotationStartServerOutput = handle_server_response(body.as_str()).unwrap();

	//fetch the key by id
	let url = get_url("api/v1/user/user_keys/key/".to_string() + key_out.key_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let _out: GroupKeyServerOutput = handle_server_response(body.as_str()).unwrap();

	sentc_crypto::user::done_key_fetch(
		&user.user_data.as_ref().unwrap().device_keys.private_key,
		body.as_str(),
	)
	.unwrap();

	//fetch new key by pagination
	let url = get_url("api/v1/user/user_keys/keys/0/none".to_string());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<GroupKeyServerOutput> = handle_server_response(body.as_str()).unwrap();

	assert_eq!(out.len(), 2);
	//keys are order by time dec
	assert_eq!(out[0].group_key_id, key_out.key_id);

	//2nd page
	let url = get_url("api/v1/user/user_keys/keys/".to_string() + out[0].time.to_string().as_str() + "/" + out[0].group_key_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out1: Vec<GroupKeyServerOutput> = handle_server_response(body.as_str()).unwrap();
	assert_eq!(out1.len(), 1);

	assert_eq!(out1[0].group_key_id, out[1].group_key_id);
}

#[tokio::test]
async fn test_27_done_key_rotation_for_other_device()
{
	//fetch user group key and done it for 2nd device

	let user = USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data_1.as_ref().unwrap().jwt; //use the jwt from the main device

	//get the data for the rotation
	let url = get_url("api/v1/user/user_keys/rotation".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<KeyRotationInput> = handle_server_response(body.as_str()).unwrap();

	let device_private_key = &user.user_data_1.as_ref().unwrap().device_keys.private_key;
	let device_public_key = &user.user_data_1.as_ref().unwrap().device_keys.public_key;
	let pre_group_key = &user.user_data_1.as_ref().unwrap().user_keys[0].group_key;

	for key in out {
		let key_id = key.new_group_key_id.clone();
		let rotation_out = sentc_crypto::group::done_key_rotation(device_private_key, device_public_key, pre_group_key, key, None).unwrap();

		//done for each key
		let url = get_url("api/v1/user/user_keys/rotation/".to_owned() + key_id.as_str());
		let client = reqwest::Client::new();
		let res = client
			.put(url)
			.header(AUTHORIZATION, auth_header(jwt))
			.header("x-sentc-app-token", &user.app_data.secret_token)
			.body(rotation_out)
			.send()
			.await
			.unwrap();

		let body = res.text().await.unwrap();
		handle_general_server_response(body.as_str()).unwrap();
	}

	let url = get_url("api/v1/user/user_keys/keys/0/none".to_string());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<GroupKeyServerOutput> = handle_server_response(body.as_str()).unwrap();

	assert_eq!(out.len(), 2);
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

	let prep_server_input = sentc_crypto::user::prepare_login_start("device_1").unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, _auth_key, _derived_master_key) = sentc_crypto::user::prepare_login("device_1", "12345", body.as_str()).unwrap();

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

	match handle_server_response::<sentc_crypto::sdk_common::user::DoneLoginServerReturn>(&body) {
		Ok(_) => panic!("should be error"),
		Err(e) => {
			match e {
				SdkError::Util(SdkUtilError::ServerErr(s, _)) => {
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

	let url = get_url("api/v1/register".to_owned());

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
	match sentc_crypto::user::done_register(body.as_str()) {
		Ok(_v) => {
			panic!("this should not be Ok")
		},
		Err(e) => {
			match e {
				SdkError::Util(SdkUtilError::ServerErr(s, m)) => {
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
async fn test_42_register_and_login_user_via_test_fn()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let secret_token = &user.app_data.secret_token;
	let public_token = &user.app_data.public_token;

	let id = register_user(secret_token, "hello", "12345").await;

	let login = login_user(public_token, "hello", "12345").await;

	assert_eq!(id, login.user_id);

	delete_user(&user.app_data.secret_token, &login.user_id).await;
}

#[tokio::test]
async fn zzz_clean_up()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let customer_jwt = &user.customer_data.verify.jwt;

	delete_app(customer_jwt, user.app_data.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use sentc_crypto::UserData;
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::user::{
	DoneLoginLightServerOutput,
	DoneLoginServerOutput,
	MasterKey,
	RegisterServerOutput,
	UserIdentifierAvailableServerInput,
	UserIdentifierAvailableServerOutput,
	UserInitServerOutput,
	UserPublicData,
	UserPublicKeyDataServerOutput,
	UserUpdateServerInput,
	UserUpdateServerOut,
	UserVerifyKeyDataServerOutput,
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
	pub user_data: Option<UserData>,
	pub app_data: AppRegisterOutput,
	pub customer_data: CustomerDoneLoginOutput,
}

static USER_TEST_STATE: OnceCell<RwLock<UserState>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_global_test()
{
	dotenv::dotenv().ok();

	let (_, customer_data) = create_test_customer("hello@test3.com", "12345").await;

	let customer_jwt = &customer_data.user_keys.jwt;

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

	assert_eq!(exists.status, true);
	assert_eq!(exists.err_code, None);

	let exists = match exists.result {
		Some(v) => v,
		None => panic!("exists is not here"),
	};

	assert_eq!(exists.user_identifier, username.to_string());
	assert_eq!(exists.available, true);
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

	assert_eq!(register_out.status, true);
	assert_eq!(register_out.err_code, None);

	let register_out = register_out.result.unwrap();
	assert_eq!(register_out.user_identifier, username.to_string());

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

	assert_eq!(exists.status, true);
	assert_eq!(exists.err_code, None);

	let exists = exists.result.unwrap();

	assert_eq!(exists.user_identifier, username.to_string());
	assert_eq!(exists.available, false);
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

	assert_eq!(error.status, false);
	assert_eq!(error.result.is_none(), true);
	assert_eq!(error.err_code.unwrap(), ApiErrorCodes::UserExists.get_int_code());

	//check err in sdk
	match sentc_crypto::user::done_register(body.as_str()) {
		Ok(_v) => {
			panic!("this should not be Ok")
		},
		Err(e) => {
			match e {
				sentc_crypto::SdkError::ServerErr(s, m) => {
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

	let (auth_key, derived_master_key) = sentc_crypto::user::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(auth_key)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let done_login = sentc_crypto::user::done_login(&derived_master_key, body.as_str()).unwrap();

	user.user_data = Some(done_login);
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

	let (auth_key, derived_master_key) = sentc_crypto::user::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(auth_key)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let login_output = ServerOutput::<DoneLoginServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(login_output.status, false);
	assert_eq!(login_output.result.is_none(), true);
	assert_eq!(login_output.err_code.unwrap(), ApiErrorCodes::Login.get_int_code());

	match sentc_crypto::user::done_login(&derived_master_key, body.as_str()) {
		Ok(_v) => {
			panic!("this should not be Ok")
		},
		Err(e) => {
			match e {
				sentc_crypto::SdkError::ServerErr(s, m) => {
					//this should be the right err
					//this are the same err as the backend
					assert_eq!(login_output.err_code.unwrap(), s);
					assert_eq!(login_output.err_msg.unwrap(), m);
				},
				_ => panic!("this should not be the right error code"),
			}
		},
	}
}

#[tokio::test]
async fn test_16_user_delete_with_wrong_jwt()
{
	//test user action with old jwt (when the jwt keys was deleted)

	//create new jwt keys for the app
	let mut user = USER_TEST_STATE.get().unwrap().write().await;

	let old_jwt = &user.user_data.as_ref().unwrap().jwt;
	let old_jwt_data = &user.app_data.jwt_data;

	let customer_jwt = &user.customer_data.user_keys.jwt;

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

	assert_eq!(delete_output.status, false);

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
	let (auth_key, derived_master_key) = sentc_crypto::user::prepare_login(username, pw, body.as_str()).unwrap();

	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
		.body(auth_key)
		.send()
		.await
		.unwrap();

	let done_body = res.text().await.unwrap();

	//2. login again to get a fresh jwt
	let done_login_out = sentc_crypto::user::done_login(&derived_master_key, done_body.as_str()).unwrap();

	let jwt = done_login_out.jwt;

	//______________________________________________________________________________________________
	//use a fresh jwt here

	let input = sentc_crypto::user::change_password(pw, new_pw, body.as_str(), done_body.as_str()).unwrap();

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

	assert_eq!(out.status, true);
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

	let (auth_key, _derived_master_key) = sentc_crypto::user::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(auth_key)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let login_output = ServerOutput::<DoneLoginServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(login_output.status, false);
	assert_eq!(login_output.result.is_none(), true);
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

	let input = sentc_crypto::user::reset_password(new_pw, &key_data.keys.private_key, &key_data.keys.sign_key).unwrap();

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

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();

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

	let (auth_key, _derived_master_key) = sentc_crypto::user::prepare_login(username, old_pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(auth_key)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let login_output = ServerOutput::<DoneLoginServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(login_output.status, false);
	assert_eq!(login_output.result.is_none(), true);
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
	let old_username = &user.username;
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

	let out = ServerOutput::<UserUpdateServerOut>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	assert_ne!(out.user_identifier, old_username.to_string());

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

	let out = ServerOutput::<UserUpdateServerOut>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, false);
	assert_eq!(out.err_code.unwrap(), 101);
}

#[tokio::test]
async fn test_21_get_user_public_data()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/user/".to_owned() + user.user_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<UserPublicData>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);

	let out = out.result.unwrap();

	assert_eq!(
		out.public_key_id,
		user.user_data
			.as_ref()
			.unwrap()
			.keys
			.public_key
			.key_id
			.to_string()
	);

	assert_eq!(
		out.verify_key_id,
		user.user_data
			.as_ref()
			.unwrap()
			.keys
			.verify_key
			.key_id
			.to_string()
	);

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

	assert_eq!(out.status, true);

	let out = out.result.unwrap();

	assert_eq!(
		out.public_key_id,
		user.user_data
			.as_ref()
			.unwrap()
			.keys
			.public_key
			.key_id
			.to_string()
	);

	//get user verify key
	let url = get_url("api/v1/user/".to_owned() + user.user_id.as_str() + "/verify_key");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<UserVerifyKeyDataServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);

	let out = out.result.unwrap();

	assert_eq!(
		out.verify_key_id,
		user.user_data
			.as_ref()
			.unwrap()
			.keys
			.verify_key
			.key_id
			.to_string()
	);
}

#[tokio::test]
async fn test_22_refresh_jwt()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt;

	let input = sentc_crypto::user::prepare_refresh_jwt(&user.user_data.as_ref().unwrap().refresh_token).unwrap();

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

	assert_eq!(out.status, true);

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

	let out = ServerOutput::<UserUpdateServerOut>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, false);
	assert_eq!(out.err_code.unwrap(), 101);

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

	assert_eq!(out.status, false);
	assert_eq!(out.err_code.unwrap(), ApiErrorCodes::WrongJwtAction.get_int_code());
}

#[tokio::test]
async fn test_23_user_normal_init()
{
	//no group invite here at this point

	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.user_data.as_ref().unwrap().jwt;

	let url = get_url("api/v1/init".to_owned());

	let input = sentc_crypto::user::prepare_refresh_jwt(&user.user_data.as_ref().unwrap().refresh_token).unwrap();

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

	assert_eq!(out.status, true);

	let out = out.result.unwrap();

	//don't save the jwt because we need a fresh jwt
	assert_eq!(out.invites.len(), 0);
}

//do user tests before this one!

#[tokio::test]
async fn test_24_user_delete()
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

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();
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
async fn test_25_not_register_user_with_wrong_input()
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

	assert_eq!(error.status, false);
	assert_eq!(error.result.is_none(), true);
	assert_eq!(error.err_code.unwrap(), ApiErrorCodes::JsonParse.get_int_code());

	//check err in sdk
	match sentc_crypto::user::done_register(body.as_str()) {
		Ok(_v) => {
			panic!("this should not be Ok")
		},
		Err(e) => {
			match e {
				sentc_crypto::SdkError::ServerErr(s, m) => {
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
async fn test_26_register_and_login_user_via_test_fn()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let secret_token = &user.app_data.secret_token;
	let public_token = &user.app_data.public_token;

	let id = register_user(secret_token, "hello", "12345").await;

	let login = login_user(public_token, "hello", "12345").await;

	assert_eq!(id, login.user_id);

	delete_user(&user.app_data.secret_token, login.jwt.as_str()).await;
}

#[tokio::test]
async fn zzz_clean_up()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let customer_jwt = &user.customer_data.user_keys.jwt;

	delete_app(customer_jwt, user.app_data.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

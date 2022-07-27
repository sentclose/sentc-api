use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use sentc_crypto::KeyData;
use sentc_crypto_common::user::{
	ChangePasswordServerOut,
	DoneLoginServerKeysOutput,
	MasterKey,
	RegisterServerOutput,
	UserDeleteServerOutput,
	UserIdentifierAvailableServerInput,
	UserIdentifierAvailableServerOutput,
	UserUpdateServerInput,
	UserUpdateServerOut,
};
use sentc_crypto_common::ServerOutput;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use server_api::core::api_res::ApiErrorCodes;
use server_api::AppRegisterOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{add_app_jwt_keys, auth_header, create_app, delete_app, delete_app_jwt_key, delete_user, get_url, login_user, register_user};

mod test_fn;

pub struct UserState
{
	pub username: String,
	pub pw: String,
	pub user_id: String,
	pub key_data: Option<KeyData>,
	pub app_data: AppRegisterOutput,
}

static USER_TEST_STATE: OnceCell<RwLock<UserState>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_global_test()
{
	//create here an app
	let app_data = create_app().await;

	//this fn must be execute first!
	USER_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(UserState {
					username: "admin_test".to_string(),
					pw: "12345".to_string(),
					user_id: "".to_string(),
					key_data: None,
					app_data,
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
		.header("x-sentc-app-token", &user.app_data.secret_token)
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
	assert_eq!(exists.available, false);
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
	assert_eq!(exists.available, true);
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

	user.key_data = Some(done_login);
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
	let login_output = ServerOutput::<DoneLoginServerKeysOutput>::from_string(body.as_str()).unwrap();

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

	let old_jwt = &user.key_data.as_ref().unwrap().jwt;
	let old_jwt_data = &user.app_data.jwt_data;

	let new_keys = add_app_jwt_keys(user.app_data.app_id.as_str()).await;

	//delete the old jwt
	delete_app_jwt_key(old_jwt_data.app_id.as_str(), old_jwt_data.jwt_id.as_str()).await;

	let url = get_url("api/v1/user".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(old_jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let delete_output = ServerOutput::<UserDeleteServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(delete_output.status, false);

	//login new in to get the new jwt with the new keys
	let username = &user.username;
	let pw = &user.pw;

	let public_token = &user.app_data.public_token;

	let login = login_user(public_token, username, pw).await;

	user.app_data.jwt_data = new_keys; //save the new jwt for other tests
	user.key_data = Some(login);
}

#[tokio::test]
async fn test_17_change_user_pw()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;
	let jwt = &user.key_data.as_ref().unwrap().jwt;
	let username = &user.username;
	let pw = &user.pw;
	let new_pw = "54321";

	let public_token = &user.app_data.public_token;

	//______________________________________________________________________________________________
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
	let (auth_key, _derived_master_key) = sentc_crypto::user::prepare_login(username, pw, body.as_str()).unwrap();

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

	//______________________________________________________________________________________________
	//use a fresh jwt here

	let input = sentc_crypto::user::change_password(pw, new_pw, body.as_str(), done_body.as_str()).unwrap();

	let url = get_url("api/v1/user/update_pw".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.body(input)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	let out = ServerOutput::<ChangePasswordServerOut>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	assert_eq!(out.user_id, user.user_id);

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
	let login_output = ServerOutput::<DoneLoginServerKeysOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(login_output.status, false);
	assert_eq!(login_output.result.is_none(), true);
	assert_eq!(login_output.err_code.unwrap(), ApiErrorCodes::Login.get_int_code());

	//______________________________________________________________________________________________
	//login with new password
	let login = login_user(public_token, username, new_pw).await;

	//the the new key data
	user.key_data = Some(login);
	user.pw = new_pw.to_string()
}

#[tokio::test]
async fn test_18_user_update()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;
	let old_username = &user.username;
	let jwt = &user.key_data.as_ref().unwrap().jwt;

	let new_username = "bla".to_string();

	let input = UserUpdateServerInput {
		user_identifier: new_username.to_string(),
	};

	let url = get_url("api/v1/user".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.body(input.to_string().unwrap())
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
async fn test_19_user_not_update_when_identifier_exists()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;
	let jwt = &user.key_data.as_ref().unwrap().jwt;

	let new_username = "bla".to_string();

	let input = UserUpdateServerInput {
		user_identifier: new_username.to_string(),
	};

	let url = get_url("api/v1/user".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.body(input.to_string().unwrap())
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

//do user tests before this one!

#[tokio::test]
async fn test_20_user_delete()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let user_id = &user.user_id;
	let jwt = &user.key_data.as_ref().unwrap().jwt;

	let url = get_url("api/v1/user".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	let delete_output = ServerOutput::<UserDeleteServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(delete_output.status, true);
	assert_eq!(delete_output.err_code, None);

	let delete_output = delete_output.result.unwrap();
	assert_eq!(delete_output.user_id, user_id.to_string());
	assert_eq!(delete_output.msg, "User deleted");

	//TODO validate it with the sdk done user delete
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
async fn test_21_not_register_user_with_wrong_input()
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
async fn test_22_register_and_login_user_via_test_fn()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let secret_token = &user.app_data.secret_token;
	let public_token = &user.app_data.public_token;

	let id = register_user(secret_token, "hello", "12345").await;

	let login = login_user(public_token, "hello", "12345").await;

	assert_eq!(id, login.user_id);

	delete_user(login.jwt.as_str(), &id).await;
}

#[tokio::test]
async fn zzz_clean_up()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;

	delete_app(user.app_data.app_id.as_str()).await;
}

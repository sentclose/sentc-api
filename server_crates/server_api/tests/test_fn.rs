#![allow(dead_code)]

use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use sentc_crypto::{KeyData, PublicKeyFormat};
use sentc_crypto_common::group::GroupCreateOutput;
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::{GroupId, ServerOutput, UserId};
use server_api::{AppDeleteOutput, AppJwtRegisterOutput, AppRegisterInput, AppRegisterOutput, JwtKeyDeleteOutput};

pub fn get_url(path: String) -> String
{
	format!("http://127.0.0.1:{}/{}", 3002, path)
}

pub fn auth_header(jwt: &str) -> String
{
	format!("Bearer {}", jwt)
}

pub async fn create_app() -> AppRegisterOutput
{
	//TODO add here the customer jwt when customer mod is done
	let url = get_url("api/v1/customer/app".to_owned());

	let input = AppRegisterInput {
		identifier: None,
	};

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<AppRegisterOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	match out.result {
		Some(v) => v,
		None => panic!("out is not here"),
	}
}

pub async fn add_app_jwt_keys(app_id: &str) -> AppJwtRegisterOutput
{
	//TODO add here the customer jwt when customer mod is done

	let url = get_url("api/v1/customer/app/".to_owned() + app_id + "/new_jwt_keys");

	let client = reqwest::Client::new();
	let res = client.patch(url).send().await.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<AppJwtRegisterOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	out.result.unwrap()
}

pub async fn delete_app_jwt_key(app_id: &str, jwt_id: &str) -> JwtKeyDeleteOutput
{
	//TODO add here the customer jwt when customer mod is done

	let url = get_url("api/v1/customer/app/".to_owned() + app_id + "/jwt/" + jwt_id);

	let client = reqwest::Client::new();
	let res = client.delete(url).send().await.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<JwtKeyDeleteOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	out.result.unwrap()
}

pub async fn delete_app(app_id: &str)
{
	//TODO add here the customer jwt when customer mod is done

	let url = get_url("api/v1/customer/app/".to_owned() + app_id);
	let client = reqwest::Client::new();
	let res = client.delete(url).send().await.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<AppDeleteOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();
	assert_eq!(out.old_app_id, app_id.to_string());
	assert_eq!(out.msg, "App deleted");
}

pub async fn register_user(app_secret_token: &str, username: &str, password: &str) -> UserId
{
	let url = get_url("api/v1/register".to_owned());

	let input = sentc_crypto::user::register(username, password).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", app_secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let user_id = sentc_crypto::user::done_register(body.as_str()).unwrap();

	assert_ne!(user_id, "".to_owned());

	user_id
}

pub async fn delete_user(app_secret_token: &str, jwt: &str)
{
	let url = get_url("api/v1/user".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", app_secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	//TODO change this to sdk done delete
	let delete_output = ServerOutput::<ServerSuccessOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(delete_output.status, true);
	assert_eq!(delete_output.err_code, None);
}

pub async fn login_user(public_token: &str, username: &str, pw: &str) -> KeyData
{
	let url = get_url("api/v1/prepare_login".to_owned());

	let prep_server_input = sentc_crypto::user::prepare_login_start(username).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token)
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
		.header("x-sentc-app-token", public_token)
		.body(auth_key)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let done_login = sentc_crypto::user::done_login(&derived_master_key, body.as_str()).unwrap();

	done_login
}

pub async fn create_test_user(secret_token: &str, public_token: &str, username: &str, pw: &str) -> (UserId, KeyData)
{
	//create test user
	let user_id = register_user(secret_token, username, pw).await;
	let key_data = login_user(public_token, username, pw).await;

	(user_id, key_data)
}

pub async fn create_group(secret_token: &str, creator_public_key: &PublicKeyFormat, parent_group_id: Option<GroupId>, jwt: &str) -> GroupId
{
	let group_input = sentc_crypto::group::prepare_create(creator_public_key, parent_group_id).unwrap();

	let url = get_url("api/v1/group".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	let out = ServerOutput::<GroupCreateOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	out.group_id
}

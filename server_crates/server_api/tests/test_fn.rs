#![allow(dead_code)]

use std::env;

use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use sentc_crypto::group::{DoneGettingGroupKeysOutput, GroupKeyData, GroupOutData};
use sentc_crypto::sdk_common::group::GroupAcceptJoinReqServerOutput;
use sentc_crypto::{PrivateKeyFormat, PublicKeyFormat, SymKeyFormat, UserData};
use sentc_crypto_common::group::{GroupCreateOutput, KeyRotationStartServerOutput};
use sentc_crypto_common::user::{RegisterData, UserInitServerOutput};
use sentc_crypto_common::{CustomerId, GroupId, ServerOutput, UserId};
use server_api_common::app::{AppFileOptions, AppJwtRegisterOutput, AppOptions, AppRegisterInput, AppRegisterOutput};
use server_api_common::customer::{CustomerDoneLoginOutput, CustomerRegisterData, CustomerRegisterOutput};

pub fn get_url(path: String) -> String
{
	format!("http://127.0.0.1:{}/{}", 3002, path)
}

pub fn auth_header(jwt: &str) -> String
{
	format!("Bearer {}", jwt)
}

/**
Register customer but without the email check
*/
pub async fn register_customer(email: String, pw: &str) -> CustomerId
{
	let url = get_url("api/v1/customer/register".to_string());

	let register_data = sentc_crypto::user::register(email.as_str(), pw).unwrap();
	let register_data = RegisterData::from_string(register_data.as_str()).unwrap();

	let public_token = env::var("SENTC_PUBLIC_TOKEN").unwrap();

	let input = CustomerRegisterData {
		email,
		register_data,
	};

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

	out.result.unwrap().customer_id
}

pub async fn login_customer(email: &str, pw: &str) -> CustomerDoneLoginOutput
{
	let public_token = env::var("SENTC_PUBLIC_TOKEN").unwrap();

	let url = get_url("api/v1/customer/prepare_login".to_owned());

	let prep_server_input = sentc_crypto::user::prepare_login_start(email).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", public_token.as_str())
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
		.header("x-sentc-app-token", public_token.as_str())
		.body(auth_key)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<CustomerDoneLoginOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	out.result.unwrap()
}

pub async fn customer_delete(customer_jwt: &str)
{
	let public_token = env::var("SENTC_PUBLIC_TOKEN").unwrap();

	let url = get_url("api/v1/customer".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header("x-sentc-app-token", public_token.as_str())
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();
}

pub async fn create_test_customer(email: &str, pw: &str) -> (CustomerId, CustomerDoneLoginOutput)
{
	let id = register_customer(email.to_string(), pw).await;
	let customer_data = login_customer(email, pw).await;

	(id, customer_data)
}

//__________________________________________________________________________________________________

pub async fn create_app(customer_jwt: &str) -> AppRegisterOutput
{
	let url = get_url("api/v1/customer/app".to_owned());

	let input = AppRegisterInput {
		identifier: None,
		options: AppOptions::default(),
		file_options: AppFileOptions::default(),
	};

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
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

pub async fn add_app_jwt_keys(customer_jwt: &str, app_id: &str) -> AppJwtRegisterOutput
{
	let url = get_url("api/v1/customer/app/".to_owned() + app_id + "/new_jwt_keys");

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<AppJwtRegisterOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	out.result.unwrap()
}

pub async fn delete_app_jwt_key(customer_jwt: &str, app_id: &str, jwt_id: &str)
{
	let url = get_url("api/v1/customer/app/".to_owned() + app_id + "/jwt/" + jwt_id);

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();
}

pub async fn delete_app(customer_jwt: &str, app_id: &str)
{
	let url = get_url("api/v1/customer/app/".to_owned() + app_id);
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();
}

//__________________________________________________________________________________________________

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

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();
}

pub async fn login_user(public_token: &str, username: &str, pw: &str) -> UserData
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

pub async fn init_user(app_secret_token: &str, jwt: &str, refresh_token: &str) -> UserInitServerOutput
{
	let url = get_url("api/v1/init".to_owned());

	let input = sentc_crypto::user::prepare_refresh_jwt(refresh_token).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", app_secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<UserInitServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);

	let out = out.result.unwrap();

	out
}

pub async fn create_test_user(secret_token: &str, public_token: &str, username: &str, pw: &str) -> (UserId, UserData)
{
	//create test user
	let user_id = register_user(secret_token, username, pw).await;
	let key_data = login_user(public_token, username, pw).await;

	(user_id, key_data)
}

pub async fn create_group(secret_token: &str, creator_public_key: &PublicKeyFormat, parent_group_id: Option<GroupId>, jwt: &str) -> GroupId
{
	let group_input = sentc_crypto::group::prepare_create(creator_public_key).unwrap();

	let url = match parent_group_id {
		Some(p) => get_url("api/v1/group".to_owned() + "/" + p.as_str() + "/child"),
		None => get_url("api/v1/group".to_owned()),
	};

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

pub async fn get_group(
	secret_token: &str,
	jwt: &str,
	group_id: &str,
	private_key: &PrivateKeyFormat,
	key_update: bool,
) -> (GroupOutData, Vec<GroupKeyData>)
{
	let url = get_url("api/v1/group/".to_owned() + group_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let data = sentc_crypto::group::get_group_data(body.as_str()).unwrap();

	let mut data_keys = Vec::with_capacity(data.keys.len());

	for key in &data.keys {
		data_keys.push(sentc_crypto::group::decrypt_group_keys(private_key, key).unwrap());
	}

	assert_eq!(data.key_update, key_update);

	(data, data_keys)
}

pub async fn add_user_by_invite(
	secret_token: &str,
	jwt: &str,
	group_id: &str,
	keys: &Vec<GroupKeyData>,
	user_to_invite_id: &str,
	user_to_invite_jwt: &str,
	user_to_add_public_key: &sentc_crypto::sdk_common::user::UserPublicKeyData,
	user_to_add_private_key: &PrivateKeyFormat,
) -> (GroupOutData, Vec<GroupKeyData>)
{
	let mut group_keys_ref = vec![];

	for decrypted_group_key in keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(user_to_add_public_key, &group_keys_ref, false).unwrap();

	let url = get_url("api/v1/group/".to_owned() + group_id + "/invite_auto/" + user_to_invite_id);

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.body(invite)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let join_res: GroupAcceptJoinReqServerOutput = sentc_crypto::util::public::handle_server_response(body.as_str()).unwrap();
	assert_eq!(join_res.session_id, None);

	//accept the invite, no need to accept the invite -> we using auto invite here

	let data = get_group(
		secret_token,
		user_to_invite_jwt,
		group_id,
		user_to_add_private_key,
		false,
	)
	.await;

	assert_eq!(data.0.rank, 4);

	data
}

pub async fn key_rotation(
	secret_token: &str,
	jwt: &str,
	group_id: &str,
	pre_group_key: &SymKeyFormat,
	invoker_public_key: &PublicKeyFormat,
	invoker_private_key: &PrivateKeyFormat,
) -> (GroupOutData, Vec<DoneGettingGroupKeysOutput>)
{
	let input = sentc_crypto::group::key_rotation(pre_group_key, invoker_public_key).unwrap();

	let url = get_url("api/v1/group/".to_owned() + group_id + "/key_rotation");
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out = ServerOutput::<KeyRotationStartServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	get_group(secret_token, jwt, out.group_id.as_str(), invoker_private_key, false).await
}

pub async fn done_key_rotation(
	secret_token: &str,
	jwt: &str,
	group_id: &str,
	pre_group_key: &SymKeyFormat,
	public_key: &PublicKeyFormat,
	private_key: &PrivateKeyFormat,
) -> Vec<DoneGettingGroupKeysOutput>
{
	//get the data for the rotation

	let url = get_url("api/v1/group/".to_owned() + group_id + "/key_rotation");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<Vec<sentc_crypto::sdk_common::group::KeyRotationInput>>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	let mut new_keys = vec![];

	//done it for each key
	for key in out {
		let rotation_out = sentc_crypto::group::done_key_rotation(private_key, public_key, pre_group_key, &key).unwrap();

		//done the key rotation to save the new key
		let url = get_url("api/v1/group/".to_owned() + group_id + "/key_rotation/" + key.new_group_key_id.as_str());
		let client = reqwest::Client::new();
		let res = client
			.put(url)
			.header(AUTHORIZATION, auth_header(jwt))
			.header("x-sentc-app-token", secret_token)
			.body(rotation_out)
			.send()
			.await
			.unwrap();

		let body = res.text().await.unwrap();
		sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();

		//fetch just the new key
		let url = get_url("api/v1/group/".to_owned() + group_id + "/key/" + key.new_group_key_id.as_str());

		let client = reqwest::Client::new();
		let res = client
			.get(url)
			.header(AUTHORIZATION, auth_header(jwt))
			.header("x-sentc-app-token", secret_token)
			.send()
			.await
			.unwrap();

		let body = res.text().await.unwrap();

		let group_key_fetch = sentc_crypto::group::get_group_key_from_server_output(body.as_str()).unwrap();

		new_keys.push(sentc_crypto::group::decrypt_group_keys(private_key, &group_key_fetch).unwrap());
	}

	new_keys
}

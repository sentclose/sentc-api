#![allow(dead_code, clippy::bool_assert_comparison)]

use std::env;
use std::time::Duration;

use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
#[cfg(feature = "mysql")]
use rustgram_server_util::db::mysql_async_export::prelude::Queryable;
use rustgram_server_util::db::StringEntity;
use sentc_crypto::group::{DoneGettingGroupKeysOutput, GroupKeyData, GroupOutData};
use sentc_crypto::sdk_common::file::FileData;
use sentc_crypto::sdk_common::group::{GroupAcceptJoinReqServerOutput, GroupHmacData, GroupInviteServerOutput};
use sentc_crypto::sdk_common::user::UserPublicKeyData;
use sentc_crypto::sdk_core::SymKey;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::util::{HmacKeyFormat, UserKeyDataInt};
use sentc_crypto::{PrivateKeyFormat, PublicKeyFormat, SdkError, SymKeyFormat, UserData};
use sentc_crypto_common::group::{GroupCreateOutput, GroupKeyServerOutput, KeyRotationStartServerOutput};
use sentc_crypto_common::user::{CaptchaCreateOutput, CaptchaInput, RegisterData, UserInitServerOutput};
use sentc_crypto_common::{CustomerId, GroupId, ServerOutput, UserId};
use server_api_common::app::{AppFileOptionsInput, AppJwtRegisterOutput, AppOptions, AppRegisterInput, AppRegisterOutput};
use server_api_common::customer::{CustomerData, CustomerDoneLoginOutput, CustomerRegisterData, CustomerRegisterOutput};

pub fn get_url(path: String) -> String
{
	format!("http://127.0.0.1:{}/{}", 3002, path)
}

pub fn get_server_error_from_normal_res(body: &str) -> u32
{
	match handle_general_server_response(body) {
		Ok(_) => panic!("should be an error"),
		Err(e) => {
			match e {
				SdkError::ServerErr(c, _) => c,
				_ => panic!("should be server error"),
			}
		},
	}
}

pub fn auth_header(jwt: &str) -> String
{
	format!("Bearer {}", jwt)
}

pub async fn get_captcha(token: &str) -> CaptchaInput
{
	//make the captcha req first
	let url = get_url("api/v1/customer/captcha".to_string());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: CaptchaCreateOutput = handle_server_response(body.as_str()).unwrap();

	//get the captcha data from the db

	//change the db path of sqlite
	dotenv::dotenv().ok();
	env::set_var("DB_PATH", env::var("DB_PATH_TEST").unwrap());

	//language=SQL
	let sql = "SELECT solution FROM sentc_captcha WHERE id = ?";

	let sol;

	/*
		Use single conn here because of the pool conn limit from mysql
	*/
	#[cfg(feature = "mysql")]
	{
		let user = env::var("DB_USER").unwrap();
		let pw = env::var("DB_PASS").unwrap();
		let mysql_host = env::var("DB_HOST").unwrap();
		let db = env::var("DB_NAME").unwrap();

		let mut conn = rustgram_server_util::db::mysql_async_export::Conn::new(
			rustgram_server_util::db::mysql_async_export::Opts::try_from(format!("mysql://{}:{}@{}/{}", user, pw, mysql_host, db).as_str()).unwrap(),
		)
		.await
		.unwrap();

		let solution: Option<StringEntity> = conn
			.exec_first(sql, rustgram_server_util::set_params!(out.captcha_id.clone()))
			.await
			.unwrap();

		sol = solution.unwrap();
	}

	#[cfg(feature = "sqlite")]
	{
		rustgram_server_util::db::init_db().await;

		let solution: Option<StringEntity> = rustgram_server_util::db::query_first(sql, rustgram_server_util::set_params!(out.captcha_id.clone()))
			.await
			.unwrap();

		sol = solution.unwrap();
	}

	CaptchaInput {
		captcha_solution: sol.0,
		captcha_id: out.captcha_id,
	}
}

/**
Register customer but without the email check
*/
pub async fn register_customer(email: String, pw: &str) -> CustomerId
{
	let public_token = env::var("SENTC_PUBLIC_TOKEN").unwrap();

	let url = get_url("api/v1/customer/register".to_string());

	let register_data = sentc_crypto::user::register(email.as_str(), pw).unwrap();
	let register_data = RegisterData::from_string(register_data.as_str()).unwrap();

	let captcha_input = get_captcha(public_token.as_str()).await;

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
		.header("x-sentc-app-token", public_token.as_str())
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

	handle_general_server_response(body.as_str()).unwrap();
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
		file_options: AppFileOptionsInput::default(),
		group_options: Default::default(),
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

	handle_general_server_response(body.as_str()).unwrap();
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

	handle_general_server_response(body.as_str()).unwrap();
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

pub async fn delete_user(app_secret_token: &str, user_id: &str)
{
	let url = get_url("api/v1/user/force/".to_owned() + user_id);
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header("x-sentc-app-token", app_secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();
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

	out.result.unwrap()
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

pub async fn create_child_group_from_group_as_member(
	secret_token: &str,
	creator_public_key: &PublicKeyFormat,
	parent_group_id: &str,
	jwt: &str,
	group_to_access: &str,
) -> GroupId
{
	let group_input = sentc_crypto::group::prepare_create(creator_public_key).unwrap();

	let url = get_url("api/v1/group".to_owned() + "/" + parent_group_id + "/child");

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", group_to_access)
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

pub fn decrypt_group_hmac_keys(first_group_key: &SymKeyFormat, hmac_keys: Vec<GroupHmacData>) -> Vec<HmacKeyFormat>
{
	//its important to use the sdk common version here and not from the api

	let mut decrypted_hmac_keys = Vec::with_capacity(hmac_keys.len());

	for hmac_key in hmac_keys {
		decrypted_hmac_keys.push(sentc_crypto::group::decrypt_group_hmac_key(first_group_key, hmac_key).unwrap());
	}

	decrypted_hmac_keys
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

	for key in data.keys {
		data_keys.push(sentc_crypto::group::decrypt_group_keys(private_key, key).unwrap());
	}

	assert_eq!(data.key_update, key_update);

	(
		GroupOutData {
			keys: vec![],
			hmac_keys: data.hmac_keys,
			parent_group_id: data.parent_group_id,
			key_update: data.key_update,
			created_time: data.created_time,
			joined_time: data.joined_time,
			rank: data.rank,
			group_id: data.group_id,
			access_by_group_as_member: data.access_by_group_as_member,
			access_by_parent_group: data.access_by_parent_group,
			is_connected_group: data.is_connected_group,
		},
		data_keys,
	)
}

pub async fn get_group_from_group_as_member(
	secret_token: &str,
	jwt: &str,
	group_id: &str,
	group_to_access: &str,
	private_group_key: &PrivateKeyFormat,
) -> (GroupOutData, Vec<GroupKeyData>)
{
	let url = get_url("api/v1/group/".to_owned() + group_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", group_to_access)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let data = sentc_crypto::group::get_group_data(body.as_str()).unwrap();

	let mut data_keys = Vec::with_capacity(data.keys.len());

	for key in data.keys {
		data_keys.push(sentc_crypto::group::decrypt_group_keys(private_group_key, key).unwrap());
	}

	(
		GroupOutData {
			keys: vec![],
			hmac_keys: data.hmac_keys,
			parent_group_id: data.parent_group_id,
			key_update: data.key_update,
			created_time: data.created_time,
			joined_time: data.joined_time,
			rank: data.rank,
			group_id: data.group_id,
			access_by_group_as_member: data.access_by_group_as_member,
			access_by_parent_group: data.access_by_parent_group,
			is_connected_group: data.is_connected_group,
		},
		data_keys,
	)
}

#[allow(clippy::too_many_arguments)]
pub async fn add_user_by_invite(
	secret_token: &str,
	jwt: &str,
	group_id: &str,
	keys: &Vec<GroupKeyData>,
	user_to_invite_id: &str,
	user_to_invite_jwt: &str,
	user_to_add_public_key: &UserPublicKeyData,
	user_to_add_private_key: &PrivateKeyFormat,
) -> (GroupOutData, Vec<GroupKeyData>)
{
	let mut group_keys_ref = vec![];

	for decrypted_group_key in keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(user_to_add_public_key, &group_keys_ref, false, None).unwrap();

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

	let join_res: GroupAcceptJoinReqServerOutput = handle_server_response(body.as_str()).unwrap();
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

#[allow(clippy::too_many_arguments)]
pub async fn add_user_by_invite_as_group_as_member(
	secret_token: &str,
	jwt: &str,
	group_id: &str,
	group_with_access: &str,
	keys: &Vec<GroupKeyData>,
	user_to_invite_id: &str,
	user_to_invite_jwt: &str,
	user_to_add_public_key: &UserPublicKeyData,
	user_to_add_private_key: &PrivateKeyFormat,
) -> (GroupOutData, Vec<GroupKeyData>)
{
	let mut group_keys_ref = vec![];

	for decrypted_group_key in keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(user_to_add_public_key, &group_keys_ref, false, None).unwrap();

	let url = get_url("api/v1/group/".to_owned() + group_id + "/invite_auto/" + user_to_invite_id);

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", group_with_access)
		.body(invite)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let join_res: GroupAcceptJoinReqServerOutput = handle_server_response(body.as_str()).unwrap();
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

#[allow(clippy::too_many_arguments)]
pub async fn add_group_by_invite(
	secret_token: &str,
	jwt: &str,
	group_id: &str,
	keys: &Vec<GroupKeyData>,
	group_to_invite_id: &str,
	group_to_invite_exported_public_key: &UserPublicKeyData,
	group_to_invite_member_jwt: &str,
	group_to_invite_private_key: &PrivateKeyFormat,
	group_as_member_id: Option<&str>,
) -> (GroupOutData, Vec<GroupKeyData>)
{
	let mut group_keys_ref = vec![];

	for decrypted_group_key in keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(group_to_invite_exported_public_key, &group_keys_ref, false, None).unwrap();

	let url = get_url("api/v1/group/".to_owned() + group_id + "/invite_group_auto/" + group_to_invite_id);

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.body(invite);

	let res = match group_as_member_id {
		Some(id) => res.header("x-sentc-group-access-id", id),
		None => res,
	};

	let res = res.send().await.unwrap();

	let body = res.text().await.unwrap();

	let invite_res: GroupInviteServerOutput = handle_server_response(&body).unwrap();

	assert_eq!(invite_res.session_id, None);

	get_group_from_group_as_member(
		secret_token,
		group_to_invite_member_jwt,
		group_id,
		group_to_invite_id,
		group_to_invite_private_key,
	)
	.await
}

pub async fn key_rotation(
	secret_token: &str,
	jwt: &str,
	group_id: &str,
	pre_group_key: &SymKeyFormat,
	invoker_public_key: &PublicKeyFormat,
	invoker_private_key: &PrivateKeyFormat,
	group_as_member_id: Option<&str>,
) -> (GroupOutData, Vec<DoneGettingGroupKeysOutput>)
{
	let input = sentc_crypto::group::key_rotation(pre_group_key, invoker_public_key, false, None, "test".to_string()).unwrap();

	let url = get_url("api/v1/group/".to_owned() + group_id + "/key_rotation");
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.body(input);

	let res = match group_as_member_id {
		Some(id) => res.header("x-sentc-group-access-id", id),
		None => res,
	};

	let res = res.send().await.unwrap();

	let body = res.text().await.unwrap();
	let out = ServerOutput::<KeyRotationStartServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	//wait a bit to finish the key rotation in the sub thread
	tokio::time::sleep(Duration::from_millis(50)).await;

	match group_as_member_id {
		Some(id) => get_group_from_group_as_member(secret_token, jwt, group_id, id, invoker_private_key).await,
		None => {
			let (one, two) = get_group(secret_token, jwt, out.group_id.as_str(), invoker_private_key, false).await;
			(one, two)
		},
	}
}

pub async fn done_key_rotation(
	secret_token: &str,
	jwt: &str,
	group_id: &str,
	pre_group_key: &SymKeyFormat,
	public_key: &PublicKeyFormat,
	private_key: &PrivateKeyFormat,
	group_as_member_id: Option<&str>,
) -> Vec<DoneGettingGroupKeysOutput>
{
	//get the data for the rotation

	let url = get_url("api/v1/group/".to_owned() + group_id + "/key_rotation");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token);

	let res = match group_as_member_id {
		Some(id) => res.header("x-sentc-group-access-id", id),
		None => res,
	};

	let res = res.send().await.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<Vec<sentc_crypto::sdk_common::group::KeyRotationInput>>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	let mut new_keys = vec![];

	//done it for each key
	for key in out {
		let key_id = key.new_group_key_id.clone();

		let rotation_out = sentc_crypto::group::done_key_rotation(private_key, public_key, pre_group_key, key, None).unwrap();

		//done the key rotation to save the new key
		let url = get_url("api/v1/group/".to_owned() + group_id + "/key_rotation/" + key_id.as_str());
		let client = reqwest::Client::new();
		let res = client
			.put(url)
			.header(AUTHORIZATION, auth_header(jwt))
			.header("x-sentc-app-token", secret_token)
			.body(rotation_out);

		let res = match group_as_member_id {
			Some(id) => res.header("x-sentc-group-access-id", id),
			None => res,
		};

		let res = res.send().await.unwrap();

		let body = res.text().await.unwrap();
		handle_general_server_response(body.as_str()).unwrap();

		//fetch just the new key
		let url = get_url("api/v1/group/".to_owned() + group_id + "/key/" + key_id.as_str());

		let client = reqwest::Client::new();
		let res = client
			.get(url)
			.header(AUTHORIZATION, auth_header(jwt))
			.header("x-sentc-app-token", secret_token);

		let res = match group_as_member_id {
			Some(id) => res.header("x-sentc-group-access-id", id),
			None => res,
		};

		let res = res.send().await.unwrap();

		let body = res.text().await.unwrap();

		let group_key_fetch = sentc_crypto::group::get_group_key_from_server_output(body.as_str()).unwrap();

		new_keys.push(sentc_crypto::group::decrypt_group_keys(private_key, group_key_fetch).unwrap());
	}

	new_keys
}

//__________________________________________________________________________________________________

pub async fn user_key_rotation(
	secret_token: &str,
	jwt: &str,
	pre_group_key: &SymKeyFormat,
	device_invoker_public_key: &PublicKeyFormat,
	device_invoker_private_key: &PrivateKeyFormat,
) -> UserKeyDataInt
{
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
		.header("x-sentc-app-token", secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let key_out: KeyRotationStartServerOutput = handle_server_response(body.as_str()).unwrap();

	//wait a bit to finish the key rotation in the sub thread
	tokio::time::sleep(Duration::from_millis(50)).await;

	//fetch the key by id

	let url = get_url("api/v1/user/user_keys/key/".to_string() + key_out.key_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let _out: GroupKeyServerOutput = handle_server_response(body.as_str()).unwrap();

	sentc_crypto::user::done_key_fetch(device_invoker_private_key, body.as_str()).unwrap()
}

//__________________________________________________________________________________________________

pub async fn get_file(file_id: &str, jwt: &str, token: &str, group_id: Option<&str>) -> FileData
{
	//download the file info
	let url = match group_id {
		Some(id) => get_url("api/v1/group/".to_string() + id + "/file/" + file_id),
		None => get_url("api/v1/file/".to_string() + file_id),
	};

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", token)
		.header(AUTHORIZATION, auth_header(jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let file_data: FileData = handle_server_response(body.as_str()).unwrap();

	file_data
}

pub async fn get_file_part(part_id: &str, jwt: &str, token: &str) -> Vec<u8>
{
	let url = get_url("api/v1/file/part/".to_string() + part_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", token)
		.header(AUTHORIZATION, auth_header(jwt))
		.send()
		.await
		.unwrap();

	if res.status() != 200 {
		let text = res.text().await.unwrap();

		panic!("error in fetching part: {:?}", text);
	}

	res.bytes().await.unwrap().to_vec()
}

pub async fn get_and_decrypt_file_part(part_id: &str, jwt: &str, token: &str, file_key: &SymKey) -> (Vec<u8>, SymKey)
{
	let buffer = get_file_part(part_id, jwt, token).await;

	sentc_crypto::file::decrypt_file_part(file_key, &buffer, None).unwrap()
}

pub async fn get_and_decrypt_file_part_start(part_id: &str, jwt: &str, token: &str, file_key: &SymKeyFormat) -> (Vec<u8>, SymKey)
{
	let buffer = get_file_part(part_id, jwt, token).await;

	sentc_crypto::file::decrypt_file_part_start(file_key, &buffer, None).unwrap()
}

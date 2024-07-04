#![allow(dead_code, clippy::bool_assert_comparison)]

use std::env;
use std::time::Duration;

use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
#[cfg(feature = "mysql")]
use rustgram_server_util::db::mysql_async_export::prelude::Queryable;
use rustgram_server_util::db::StringEntity;
use sentc_crypto::entities::group::GroupOutData;
use sentc_crypto::group;
use sentc_crypto::sdk_common::file::FileData;
use sentc_crypto::sdk_common::group::{GroupHmacData, GroupSortableData};
use sentc_crypto::sdk_common::user::UserPublicKeyData;
use sentc_crypto::sdk_utils::error::SdkUtilError;
use sentc_crypto::sdk_utils::{handle_general_server_response, handle_server_response};
use sentc_crypto::util_req_full::user::PreLoginOut;
use sentc_crypto_common::group::{GroupKeyServerOutput, KeyRotationStartServerOutput};
use sentc_crypto_common::user::{CaptchaCreateOutput, CaptchaInput, UserDeviceRegisterInput, UserForcedAction};
use sentc_crypto_common::{CustomerId, GroupId, ServerOutput, UserId};
use server_dashboard_common::app::{AppFileOptionsInput, AppJwtRegisterOutput, AppOptions, AppRegisterInput, AppRegisterOutput};
use server_dashboard_common::customer::{CustomerData, CustomerDoneLoginOutput, CustomerRegisterData, CustomerRegisterOutput};

#[cfg(feature = "std_keys")]
pub type TestUser = sentc_crypto::keys::std::StdUser;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestUser = sentc_crypto::keys::fips::FipsUser;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestUser = sentc_crypto::keys::rec::RecUser;

#[cfg(feature = "std_keys")]
pub type TestUserDataInt = sentc_crypto::keys::std::StdUserDataInt;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestUserDataInt = sentc_crypto::keys::fips::FipsUserDataInt;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestUserDataInt = sentc_crypto::keys::rec::RecUserDataInt;

#[cfg(feature = "std_keys")]
pub type TestUserKeyDataInt = sentc_crypto::keys::std::StdUserKeyDataInt;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestUserKeyDataInt = sentc_crypto::keys::fips::FipsUserKeyDataInt;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestUserKeyDataInt = sentc_crypto::keys::rec::RecUserKeyDataInt;

#[cfg(feature = "std_keys")]
pub type TestGroup = sentc_crypto::keys::std::StdGroup;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestGroup = sentc_crypto::keys::fips::FipsGroup;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestGroup = sentc_crypto::keys::rec::RecGroup;

#[cfg(feature = "std_keys")]
pub type TestFileEncryptor = sentc_crypto::keys::std::StdFileEncryptor;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestFileEncryptor = sentc_crypto::keys::fips::FipsFileEncryptor;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestFileEncryptor = sentc_crypto::keys::rec::RecFileEncryptor;

#[cfg(feature = "std_keys")]
pub type TestKeyGenerator = sentc_crypto::keys::std::StdKeyGenerator;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestKeyGenerator = sentc_crypto::keys::fips::FipsKeyGenerator;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestKeyGenerator = sentc_crypto::keys::rec::RecKeyGenerator;

#[cfg(feature = "std_keys")]
pub type TestGroupKeyData = sentc_crypto::keys::std::StdGroupKeyData;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestGroupKeyData = sentc_crypto::keys::fips::FipsGroupKeyData;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestGroupKeyData = sentc_crypto::keys::rec::RecGroupKeyData;

#[cfg(feature = "std_keys")]
pub type TestPublicKey = sentc_crypto::std_keys::util::PublicKey;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestPublicKey = sentc_crypto::fips_keys::util::PublicKey;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestPublicKey = sentc_crypto::rec_keys::util::PublicKey;

#[cfg(feature = "std_keys")]
pub type TestSymmetricKey = sentc_crypto::std_keys::util::SymmetricKey;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestSymmetricKey = sentc_crypto::fips_keys::util::SymmetricKey;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestSymmetricKey = sentc_crypto::rec_keys::util::SymmetricKey;

#[cfg(feature = "std_keys")]
pub type TestSecretKey = sentc_crypto::std_keys::util::SecretKey;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestSecretKey = sentc_crypto::fips_keys::util::SecretKey;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestSecretKey = sentc_crypto::rec_keys::util::SecretKey;

#[cfg(feature = "std_keys")]
pub type TestHmacKey = sentc_crypto_std_keys::util::HmacKey;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestHmacKey = sentc_crypto::fips_keys::util::HmacKey;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestHmacKey = sentc_crypto::rec_keys::util::HmacKey;

#[cfg(feature = "std_keys")]
pub type TestSortableKey = sentc_crypto::std_keys::util::SortableKey;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type TestSortableKey = sentc_crypto::fips_keys::util::SortableKey;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type TestSortableKey = sentc_crypto::rec_keys::util::SortableKey;

#[cfg(feature = "std_keys")]
pub type CoreSymmetricKey = sentc_crypto::std_keys::core::SymmetricKey;
#[cfg(all(feature = "fips_keys", not(feature = "std_keys")))]
pub type CoreSymmetricKey = sentc_crypto::fips_keys::core::sym::Aes256GcmKey;
#[cfg(all(feature = "rec_keys", not(feature = "std_keys")))]
pub type CoreSymmetricKey = sentc_crypto::rec_keys::core::sym::Aes256GcmKey;

pub fn get_base_url() -> String
{
	format!("http://127.0.0.1:{}", 3002)
}

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
				SdkUtilError::ServerErr(c, _) => c,
				_ => panic!("should be server error"),
			}
		},
	}
}

pub fn auth_header(jwt: &str) -> String
{
	format!("Bearer {}", jwt)
}

pub async fn get_captcha() -> CaptchaInput
{
	//make the captcha req first
	let url = get_url("api/v1/customer/captcha".to_string());

	let client = reqwest::Client::new();
	let res = client.get(url).send().await.unwrap();

	let body = res.text().await.unwrap();

	let out: CaptchaCreateOutput = handle_server_response(body.as_str()).unwrap();

	//get the captcha data from the db

	//change the db path of sqlite
	dotenv::from_filename("sentc.env").ok();
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
	let url = get_url("api/v1/customer/register".to_string());

	let register_data = sentc_crypto_light::user::register(email.as_str(), pw).unwrap();
	let register_data: UserDeviceRegisterInput = serde_json::from_str(register_data.as_str()).unwrap();

	let captcha_input = get_captcha().await;

	let input = CustomerRegisterData {
		customer_data: CustomerData {
			name: "abc".to_string(),
			first_name: "abc".to_string(),
			company: None,
		},
		email,
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

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	out.result.unwrap().customer_id
}

pub async fn login_customer(email: &str, pw: &str) -> CustomerDoneLoginOutput
{
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
		sentc_crypto_light::sdk_common::user::DoneLoginServerReturn::Direct(d) => {
			let keys = sentc_crypto_light::user::done_login(&derived_master_key, auth_key, email.to_string(), d).unwrap();

			let url = get_url("api/v1/customer/verify_login".to_owned());
			let client = reqwest::Client::new();
			let res = client.post(url).body(keys.challenge).send().await.unwrap();

			let server_out = res.text().await.unwrap();

			let server_out: CustomerDoneLoginOutput = handle_server_response(&server_out).unwrap();

			server_out
		},
		sentc_crypto_light::sdk_common::user::DoneLoginServerReturn::Otp => {
			panic!("No mfa excepted for customer login")
		},
	}
}

pub async fn customer_delete(customer_jwt: &str)
{
	let url = get_url("api/v1/customer".to_owned());

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
	TestUser::register_req(get_base_url(), app_secret_token, username, password)
		.await
		.unwrap()
}

pub async fn delete_user(app_secret_token: &str, user_identifier: String)
{
	let input = UserForcedAction {
		user_identifier,
	};

	let url = get_url("api/v1/user/forced/delete".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", app_secret_token)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();
}

pub async fn login_user_light(public_token: &str, username: &str, pw: &str) -> sentc_crypto_light::UserDataInt
{
	let out = sentc_crypto_light::util_req_full::user::login(get_base_url(), public_token, username, pw)
		.await
		.unwrap();

	if let sentc_crypto_light::util_req_full::user::PreLoginOut::Direct(d) = out {
		d
	} else {
		panic!("No mfa excepted");
	}
}

pub async fn login_user(public_token: &str, username: &str, pw: &str) -> TestUserDataInt
{
	let out = TestUser::login(get_base_url(), public_token, username, pw)
		.await
		.unwrap();

	if let PreLoginOut::<_, _, _, _, _, _>::Direct(d) = out {
		d
	} else {
		panic!("No mfa excepted");
	}
}

pub async fn init_user(app_secret_token: &str, jwt: &str, refresh_token: &str) -> sentc_crypto::sdk_common::user::UserInitServerOutput
{
	sentc_crypto::util_req_full::user::init_user(get_base_url(), app_secret_token, jwt, refresh_token.to_string())
		.await
		.unwrap()
}

pub async fn create_test_user(secret_token: &str, public_token: &str, username: &str, pw: &str) -> (UserId, TestUserDataInt)
{
	//create test user
	let user_id = register_user(secret_token, username, pw).await;
	let key_data = login_user(public_token, username, pw).await;

	(user_id, key_data)
}

pub async fn create_group(secret_token: &str, creator_public_key: &TestPublicKey, parent_group_id: Option<GroupId>, jwt: &str) -> GroupId
{
	match parent_group_id {
		Some(i) => {
			TestGroup::create_child_group(get_base_url(), secret_token, jwt, &i, 0, creator_public_key, None)
				.await
				.unwrap()
		},
		None => {
			TestGroup::create(get_base_url(), secret_token, jwt, creator_public_key, None)
				.await
				.unwrap()
		},
	}
}

pub async fn create_child_group_from_group_as_member(
	secret_token: &str,
	creator_public_key: &TestPublicKey,
	parent_group_id: &str,
	jwt: &str,
	group_to_access: &str,
) -> GroupId
{
	TestGroup::create_child_group(
		get_base_url(),
		secret_token,
		jwt,
		parent_group_id,
		0,
		creator_public_key,
		Some(group_to_access),
	)
	.await
	.unwrap()
}

pub fn decrypt_group_hmac_keys(first_group_key: &TestSymmetricKey, hmac_keys: Vec<GroupHmacData>) -> Vec<TestHmacKey>
{
	//it's important to use the sdk common version here and not from the api

	let mut decrypted_hmac_keys = Vec::with_capacity(hmac_keys.len());

	for hmac_key in hmac_keys {
		decrypted_hmac_keys.push(TestGroup::decrypt_group_hmac_key(first_group_key, hmac_key).unwrap());
	}

	decrypted_hmac_keys
}

pub fn decrypt_group_sortable_keys(first_group_key: &TestSymmetricKey, keys: Vec<GroupSortableData>) -> Vec<TestSortableKey>
{
	//it's important to use the sdk common version here and not from the api

	let mut decrypted_keys = Vec::with_capacity(keys.len());

	for key in keys {
		decrypted_keys.push(TestGroup::decrypt_group_sortable_key(first_group_key, key).unwrap());
	}

	decrypted_keys
}

pub async fn get_group(
	secret_token: &str,
	jwt: &str,
	group_id: &str,
	private_key: &TestSecretKey,
	key_update: bool,
) -> (GroupOutData, Vec<TestGroupKeyData>)
{
	let data = sentc_crypto::util_req_full::group::get_group(get_base_url(), secret_token, jwt, group_id, None)
		.await
		.unwrap();

	let data_keys = data
		.keys
		.into_iter()
		.map(|k| TestGroup::decrypt_group_keys(private_key, k).unwrap())
		.collect();

	assert_eq!(data.key_update, key_update);

	(
		GroupOutData {
			keys: vec![],
			hmac_keys: data.hmac_keys,
			sortable_keys: data.sortable_keys,
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
	private_group_key: &TestSecretKey,
) -> (GroupOutData, Vec<TestGroupKeyData>)
{
	let data = sentc_crypto::util_req_full::group::get_group(get_base_url(), secret_token, jwt, group_id, Some(group_to_access))
		.await
		.unwrap();

	let data_keys = data
		.keys
		.into_iter()
		.map(|k| TestGroup::decrypt_group_keys(private_group_key, k).unwrap())
		.collect();

	(
		GroupOutData {
			keys: vec![],
			hmac_keys: data.hmac_keys,
			sortable_keys: data.sortable_keys,
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
	keys: &Vec<TestGroupKeyData>,
	user_to_invite_id: &str,
	user_to_invite_jwt: &str,
	user_to_add_public_key: &UserPublicKeyData,
	user_to_add_private_key: &TestSecretKey,
) -> (GroupOutData, Vec<TestGroupKeyData>)
{
	let mut group_keys_ref = vec![];

	for decrypted_group_key in keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let join_res = TestGroup::invite_user(
		get_base_url(),
		secret_token,
		jwt,
		group_id,
		user_to_invite_id,
		1,
		None,
		1,
		true,
		false,
		false,
		user_to_add_public_key,
		&group_keys_ref,
		None,
	)
	.await
	.unwrap();

	assert_eq!(join_res, None);

	//no need to accept the invite -> we're using auto invite here

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
	keys: &Vec<TestGroupKeyData>,
	user_to_invite_id: &str,
	user_to_invite_jwt: &str,
	user_to_add_public_key: &UserPublicKeyData,
	user_to_add_private_key: &TestSecretKey,
) -> (GroupOutData, Vec<TestGroupKeyData>)
{
	let mut group_keys_ref = vec![];

	for decrypted_group_key in keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let join_res = TestGroup::invite_user(
		get_base_url(),
		secret_token,
		jwt,
		group_id,
		user_to_invite_id,
		1,
		None,
		1,
		true,
		false,
		false,
		user_to_add_public_key,
		&group_keys_ref,
		Some(group_with_access),
	)
	.await
	.unwrap();

	assert_eq!(join_res, None);

	//no need to accept the invite -> we're using auto invite here

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
	keys: &Vec<TestGroupKeyData>,
	group_to_invite_id: &str,
	group_to_invite_exported_public_key: &UserPublicKeyData,
	group_to_invite_member_jwt: &str,
	group_to_invite_private_key: &TestSecretKey,
	group_as_member_id: Option<&str>,
) -> (GroupOutData, Vec<TestGroupKeyData>)
{
	let mut group_keys_ref = vec![];

	for decrypted_group_key in keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let join_res = TestGroup::invite_user(
		get_base_url(),
		secret_token,
		jwt,
		group_id,
		group_to_invite_id,
		1,
		None,
		1,
		true,
		true,
		false,
		group_to_invite_exported_public_key,
		&group_keys_ref,
		group_as_member_id,
	)
	.await
	.unwrap();

	assert_eq!(join_res, None);

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
	pre_group_key: &TestSymmetricKey,
	invoker_public_key: &TestPublicKey,
	invoker_private_key: &TestSecretKey,
	group_as_member_id: Option<&str>,
) -> (GroupOutData, Vec<TestGroupKeyData>)
{
	let input = TestGroup::key_rotation(pre_group_key, invoker_public_key, false, None, "test".to_string()).unwrap();

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
	tokio::time::sleep(Duration::from_millis(100)).await;

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
	pre_group_key: &TestSymmetricKey,
	public_key: &TestPublicKey,
	private_key: &TestSecretKey,
	group_as_member_id: Option<&str>,
) -> Vec<TestGroupKeyData>
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

		let rotation_out = TestGroup::done_key_rotation(private_key, public_key, pre_group_key, key, None).unwrap();

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

		let group_key_fetch = group::get_group_key_from_server_output(body.as_str()).unwrap();

		new_keys.push(TestGroup::decrypt_group_keys(private_key, group_key_fetch).unwrap());
	}

	new_keys
}

//__________________________________________________________________________________________________

pub async fn user_key_rotation(
	secret_token: &str,
	jwt: &str,
	pre_group_key: &TestSymmetricKey,
	device_invoker_public_key: &TestPublicKey,
	device_invoker_private_key: &TestSecretKey,
) -> TestUserKeyDataInt
{
	let key_id = TestUser::key_rotation(
		get_base_url(),
		secret_token,
		jwt,
		device_invoker_public_key,
		pre_group_key,
	)
	.await
	.unwrap();

	//wait a bit to finish the key rotation in the sub thread
	tokio::time::sleep(Duration::from_millis(60)).await;

	//fetch the key by id

	let url = get_url("api/v1/user/user_keys/key/".to_string() + &key_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let _out: GroupKeyServerOutput = handle_server_response(&body).unwrap();

	TestUser::done_key_fetch(device_invoker_private_key, &body).unwrap()
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

pub async fn get_and_decrypt_file_part(part_id: &str, jwt: &str, token: &str, file_key: &CoreSymmetricKey) -> (Vec<u8>, CoreSymmetricKey)
{
	let buffer = get_file_part(part_id, jwt, token).await;

	TestFileEncryptor::decrypt_file_part(file_key, &buffer, None).unwrap()
}

pub async fn get_and_decrypt_file_part_start(part_id: &str, jwt: &str, token: &str, file_key: &TestSymmetricKey) -> (Vec<u8>, CoreSymmetricKey)
{
	let buffer = get_file_part(part_id, jwt, token).await;

	TestFileEncryptor::decrypt_file_part_start(file_key, &buffer, None).unwrap()
}

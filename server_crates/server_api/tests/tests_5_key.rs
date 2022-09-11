use reqwest::header::AUTHORIZATION;
use sentc_crypto::group::GroupKeyData;
use sentc_crypto::sdk_common::crypto::GeneratedSymKeyHeadServerOutput;
use sentc_crypto::sdk_common::ServerOutput;
use sentc_crypto::{SymKeyFormat, UserData};
use sentc_crypto_common::crypto::GeneratedSymKeyHeadServerRegisterOutput;
use sentc_crypto_common::SymKeyId;
use server_api_common::app::AppRegisterOutput;
use server_api_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{auth_header, create_app, create_group, create_test_customer, create_test_user, customer_delete, get_group, get_url};

mod test_fn;

pub struct SymKeyData
{
	pub id: SymKeyId,
	pub server_out: Option<GeneratedSymKeyHeadServerOutput>,
	pub sym_key: Option<SymKeyFormat>,
}

pub struct KeyState
{
	pub user_data: UserData,
	pub username: String,
	pub user_pw: String,
	pub app_data: AppRegisterOutput,
	pub customer_data: CustomerDoneLoginOutput,
	pub group_keys: Vec<GroupKeyData>,
	pub keys: Vec<SymKeyData>,
}

static KEY_TEST_STATE: OnceCell<RwLock<KeyState>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_global_state()
{
	dotenv::dotenv().ok();

	let (_, customer_data) = create_test_customer("hello@test5.com", "12345").await;

	let customer_jwt = &customer_data.user_keys.jwt;

	//create here an app
	let app_data = create_app(customer_jwt).await;

	let secret_token = app_data.secret_token.to_string();
	let public_token = app_data.public_token.to_string();

	let user_pw = "12345";
	let username = "hello5";

	let (_user_id, key_data) = create_test_user(secret_token.as_str(), public_token.as_str(), username, user_pw).await;

	let group_id = create_group(
		secret_token.as_str(),
		&key_data.user_keys[0].public_key,
		None,
		key_data.jwt.as_str(),
	)
	.await;

	let group_keys = get_group(
		secret_token.as_str(),
		key_data.jwt.as_str(),
		group_id.as_str(),
		&key_data.user_keys[0].private_key,
		false,
	)
	.await
	.1;

	KEY_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(KeyState {
					user_data: key_data,
					username: username.to_string(),
					user_pw: user_pw.to_string(),
					app_data,
					customer_data,
					group_keys,
					keys: vec![],
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_10_create_key()
{
	let mut state = KEY_TEST_STATE.get().unwrap().write().await;

	let key_data = sentc_crypto::crypto::prepare_register_sym_key(&state.group_keys[0].group_key).unwrap();

	let url = get_url("api/v1/keys/sym_key".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.header("x-sentc-app-token", state.app_data.secret_token.as_str())
		.body(key_data.0)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<GeneratedSymKeyHeadServerRegisterOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	state.keys.push(SymKeyData {
		id: out.key_id,
		server_out: None,
		sym_key: None,
	});
}

#[tokio::test]
async fn test_11_get_key_by_id()
{
	let mut state = KEY_TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/keys/sym_key/".to_owned() + state.keys[0].id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<GeneratedSymKeyHeadServerOutput>::from_string(body.as_str()).unwrap();
	let out = out.result.unwrap();

	let sym_key = sentc_crypto::crypto::done_fetch_sym_key(&state.group_keys[0].group_key, body.as_str()).unwrap();

	//test the key
	let text = "hello";

	let encrypted = sentc_crypto::crypto::encrypt_string_symmetric(&sym_key, text, None).unwrap();

	let decrypted = sentc_crypto::crypto::decrypt_string_symmetric(&sym_key, &encrypted, None).unwrap();

	assert_eq!(decrypted, text);

	let key_data = SymKeyData {
		id: out.key_id.to_string(),
		server_out: Some(out),
		sym_key: Some(sym_key),
	};

	state.keys[0] = key_data;
}

#[tokio::test]
async fn test_12_create_second_key_for_pagination()
{
	let mut state = KEY_TEST_STATE.get().unwrap().write().await;

	let key_data = sentc_crypto::crypto::prepare_register_sym_key(&state.group_keys[0].group_key).unwrap();

	let url = get_url("api/v1/keys/sym_key".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.header("x-sentc-app-token", state.app_data.secret_token.as_str())
		.body(key_data.0)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<GeneratedSymKeyHeadServerRegisterOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	state.keys.push(SymKeyData {
		id: out.key_id,
		server_out: None,
		sym_key: None,
	});
}

#[tokio::test]
async fn test_13_get_key_from_master_key()
{
	let state = KEY_TEST_STATE.get().unwrap().read().await;

	let master_key = &state.group_keys[0].group_key;

	let url = get_url("api/v1/keys/sym_key/master_key/".to_owned() + master_key.key_id.as_str() + "/0" + "/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<Vec<GeneratedSymKeyHeadServerOutput>>::from_string(body.as_str()).unwrap();
	let out = out.result.unwrap();

	assert_eq!(out.len(), 2);

	//order by id dec
	assert_eq!(out[0].key_id.to_string(), state.keys[1].id.to_string());
	assert_eq!(out[1].key_id.to_string(), state.keys[0].id.to_string());

	let sym_keys = sentc_crypto::crypto::done_fetch_sym_keys(&state.group_keys[0].group_key, body.as_str()).unwrap();

	//test the key
	let text = "hello";

	for sym_key in sym_keys.0 {
		let encrypted = sentc_crypto::crypto::encrypt_string_symmetric(&sym_key, text, None).unwrap();

		let decrypted = sentc_crypto::crypto::decrypt_string_symmetric(&sym_key, &encrypted, None).unwrap();

		assert_eq!(decrypted, text);
	}
}

#[tokio::test]
async fn test_14_delete_key()
{
	let state = KEY_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/keys/sym_key/".to_owned() + state.keys[0].id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	sentc_crypto::util::public::handle_general_server_response(body.as_str()).unwrap();
}

#[tokio::test]
async fn zz_clean_up()
{
	let state = KEY_TEST_STATE.get().unwrap().read().await;

	customer_delete(state.customer_data.user_keys.jwt.as_str()).await;
}

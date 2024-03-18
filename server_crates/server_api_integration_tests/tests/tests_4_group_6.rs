//Force group fn

use reqwest::header::AUTHORIZATION;
use sentc_crypto::entities::group::GroupKeyData;
use sentc_crypto::entities::user::UserDataInt as UserDataIntFull;
use sentc_crypto::sdk_utils::error::SdkUtilError;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::SdkError;
use sentc_crypto_common::group::{GroupCreateOutput, GroupLightServerData};
use sentc_crypto_common::{GroupId, UserId};
use sentc_crypto_light::UserDataInt as UserDataIntLight;
use server_dashboard_common::app::AppRegisterOutput;
use server_dashboard_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
	auth_header,
	create_app,
	create_test_customer,
	create_test_user,
	customer_delete,
	delete_app,
	delete_user,
	get_base_url,
	get_group,
	get_group_from_group_as_member,
	get_url,
};

mod test_fn;

pub struct UserState
{
	pub username: String,
	pub pw: String,
	pub user_id: UserId,
	pub user_data: UserDataIntFull,
}

pub struct GroupState
{
	pub group_id: GroupId,
	pub decrypted_group_keys: Vec<GroupKeyData>,
}

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerDoneLoginOutput>> = OnceCell::const_new();
static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<Vec<UserState>>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<GroupState>> = OnceCell::const_new();

pub struct UserStateLight
{
	pub username: String,
	pub pw: String,
	pub user_id: UserId,
	pub user_data: UserDataIntLight,
}

pub struct GroupStateLight
{
	pub group_id: GroupId,
}

static USERS_TEST_STATE_LIGHT: OnceCell<RwLock<Vec<UserStateLight>>> = OnceCell::const_new();
static GROUP_TEST_STATE_LIGHT: OnceCell<RwLock<GroupStateLight>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_global_test()
{
	dotenv::from_filename("sentc.env").ok();

	let (_, customer_data) = create_test_customer("helle@test4.com", "12345").await;

	let customer_jwt = customer_data.verify.jwt.to_string();

	CUSTOMER_TEST_STATE
		.get_or_init(|| async move { RwLock::new(customer_data) })
		.await;

	//create here an app
	let app_data = create_app(customer_jwt.as_str()).await;

	let secret_token = app_data.secret_token.to_string();
	let public_token = app_data.public_token.to_string();

	APP_TEST_STATE
		.get_or_init(|| async move { RwLock::new(app_data) })
		.await;

	let mut users = vec![];

	let user_pw = "12345";

	let secret_token_str = secret_token.as_str();
	let public_token_str = public_token.as_str();

	for i in 0..1 {
		let username = "hi".to_string() + i.to_string().as_str();

		let (user_id, key_data) = create_test_user(secret_token_str, public_token_str, username.as_str(), user_pw).await;

		let user = UserState {
			username,
			pw: user_pw.to_string(),
			user_id,
			user_data: key_data,
		};

		users.push(user);
	}

	USERS_TEST_STATE
		.get_or_init(|| async move { RwLock::new(users) })
		.await;

	//light users

	let mut users = vec![];

	for i in 0..1 {
		let username = "hi1".to_string() + i.to_string().as_str();

		let user_id = sentc_crypto_light_full::user::register(get_base_url(), secret_token_str, &username, user_pw)
			.await
			.unwrap();

		let out = sentc_crypto_light_full::user::login(get_base_url(), public_token_str, &username, user_pw)
			.await
			.unwrap();

		let key_data = if let sentc_crypto_light_full::user::PreLoginOut::Direct(d) = out {
			d
		} else {
			panic!("No mfa excepted");
		};

		let user = UserStateLight {
			username,
			pw: user_pw.to_string(),
			user_id,
			user_data: key_data,
		};

		users.push(user);
	}

	USERS_TEST_STATE_LIGHT
		.get_or_init(|| async move { RwLock::new(users) })
		.await;
}

//__________________________________________________________________________________________________

#[tokio::test]
async fn test_10_create_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let group_input = sentc_crypto::group::prepare_create(&creator.user_data.user_keys[0].public_key).unwrap();

	let url = get_url("api/v1/group/forced/".to_owned() + &creator.user_id);
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &out_1.group_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let data = sentc_crypto::group::get_group_data(body.as_str()).unwrap();

	let mut decrypted_group_keys = Vec::with_capacity(data.keys.len());

	for key in data.keys {
		decrypted_group_keys.push(sentc_crypto::group::decrypt_group_keys(&creator.user_data.user_keys[0].private_key, key).unwrap());
	}

	GROUP_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(GroupState {
					group_id: out_1.group_id,
					decrypted_group_keys,
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_10_delete_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let mut group = GROUP_TEST_STATE.get().unwrap().write().await;
	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let client = reqwest::Client::new();
	let res = client
		.delete(get_url("api/v1/group/forced/".to_owned() + &group.group_id))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//fetch the group
	let url = get_url("api/v1/group/".to_owned() + &group.group_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	match sentc_crypto::group::get_group_data(body.as_str()) {
		Err(SdkError::Util(SdkUtilError::ServerErr(s, _))) => {
			assert_eq!(s, 310);
		},
		_ => {
			panic!("Must be error")
		},
	}

	//now create the group again for the other tests
	let group_input = sentc_crypto::group::prepare_create(&creator.user_data.user_keys[0].public_key).unwrap();

	let url = get_url("api/v1/group/forced/".to_owned() + &creator.user_id);
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &out_1.group_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let data = sentc_crypto::group::get_group_data(body.as_str()).unwrap();

	let mut decrypted_group_keys = Vec::with_capacity(data.keys.len());

	for key in data.keys {
		decrypted_group_keys.push(sentc_crypto::group::decrypt_group_keys(&creator.user_data.user_keys[0].private_key, key).unwrap());
	}

	group.group_id = out_1.group_id;
	group.decrypted_group_keys = decrypted_group_keys;
}

#[tokio::test]
async fn test_11_create_light_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let creator = USERS_TEST_STATE_LIGHT.get().unwrap().read().await;
	let creator = &creator[0];

	let client = reqwest::Client::new();
	let res = client
		.post(get_url(
			"api/v1/group/forced/".to_owned() + &creator.user_id + "/light",
		))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &out_1.group_id + "/light");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let _out: GroupLightServerData = handle_server_response(&body).unwrap();

	GROUP_TEST_STATE_LIGHT
		.get_or_init(|| {
			async move {
				RwLock::new(GroupStateLight {
					group_id: out_1.group_id,
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_13_create_child_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	//use here the public group key for child group!
	let group_public_key = &group.decrypted_group_keys[0].public_group_key;
	let group_private_key = &group.decrypted_group_keys[0].private_group_key;

	let group_input = sentc_crypto::group::prepare_create(group_public_key).unwrap();

	let url = get_url("api/v1/group/forced/".to_owned() + &creator.user_id + "/" + &group.group_id + "/child");
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let (_data, _keys) = get_group(
		secret_token,
		creator.user_data.jwt.as_str(),
		&out_1.group_id,
		group_private_key,
		false,
	)
	.await;
}

#[tokio::test]
async fn test_14_create_child_group_light()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE_LIGHT.get().unwrap().read().await;

	let creator = USERS_TEST_STATE_LIGHT.get().unwrap().read().await;
	let creator = &creator[0];

	let client = reqwest::Client::new();
	let res = client
		.post(get_url(
			"api/v1/group/forced/".to_owned() + &creator.user_id + "/" + &group.group_id + "/child/light",
		))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &out_1.group_id + "/light");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let _out: GroupLightServerData = handle_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_15_create_connected_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	//use here the public group key for child group!
	let group_public_key = &group.decrypted_group_keys[0].public_group_key;
	let group_private_key = &group.decrypted_group_keys[0].private_group_key;

	let group_input = sentc_crypto::group::prepare_create(group_public_key).unwrap();

	let url = get_url("api/v1/group/forced/".to_owned() + &creator.user_id + "/" + &group.group_id + "/connected");
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let (_data, _keys) = get_group_from_group_as_member(
		secret_token,
		creator.user_data.jwt.as_str(),
		&out_1.group_id,
		&group.group_id,
		group_private_key,
	)
	.await;
}

#[tokio::test]
async fn test_16_create_connected_group_light()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE_LIGHT.get().unwrap().read().await;

	let creator = USERS_TEST_STATE_LIGHT.get().unwrap().read().await;
	let creator = &creator[0];

	let url = get_url("api/v1/group/forced/".to_owned() + &creator.user_id + "/" + &group.group_id + "/connected/light");
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &out_1.group_id + "/light");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &group.group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let _out: GroupLightServerData = handle_server_response(&body).unwrap();
}

//__________________________________________________________________________________________________

#[tokio::test]
async fn zzz_clean_up()
{
	let app = APP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	for user in users.iter() {
		delete_user(secret_token, user.username.clone()).await;
	}

	let users = USERS_TEST_STATE_LIGHT.get().unwrap().read().await;

	for user in users.iter() {
		delete_user(secret_token, user.username.clone()).await;
	}

	let customer_jwt = &CUSTOMER_TEST_STATE.get().unwrap().read().await.verify.jwt;

	delete_app(customer_jwt, app.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

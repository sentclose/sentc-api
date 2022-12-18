use std::collections::HashMap;

use reqwest::header::AUTHORIZATION;
use sentc_crypto::group::GroupKeyData;
use sentc_crypto::util::public::handle_server_response;
use sentc_crypto::UserData;
use sentc_crypto_common::content::{ContentCreateOutput, ListContentItem};
use sentc_crypto_common::{ContentId, GroupId, UserId};
use server_api_common::app::AppRegisterOutput;
use server_api_common::customer::CustomerDoneLoginOutput;
use server_core::input_helper::json_to_string;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
	auth_header,
	create_app,
	create_group,
	create_test_customer,
	create_test_user,
	customer_delete,
	delete_app,
	delete_user,
	get_group,
	get_url,
};

mod test_fn;

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerDoneLoginOutput>> = OnceCell::const_new();
static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<Vec<UserState>>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();
static CHILD_GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();
static CONTENT_TEST_STATE: OnceCell<RwLock<Vec<ContentState>>> = OnceCell::const_new();

pub struct UserState
{
	pub username: String,
	pub pw: String,
	pub user_id: UserId,
	pub user_data: UserData,
}

pub struct GroupState
{
	pub group_id: GroupId,
	pub group_member: Vec<UserId>,
	pub decrypted_group_keys: HashMap<UserId, Vec<GroupKeyData>>,
}

pub struct ContentState
{
	pub id: ContentId,
	pub content: String,
}

#[tokio::test]
async fn aaa_init_global_test()
{
	dotenv::dotenv().ok();

	let (_, customer_data) = create_test_customer("helle@test4.com", "12345").await;

	let customer_jwt = customer_data.user_keys.jwt.to_string();

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

	for i in 0..2 {
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

	let n = 2;
	let mut groups = Vec::with_capacity(n);

	#[allow(clippy::needless_range_loop)]
	for i in 0..n {
		let user = &users[i];
		let public_key = &user.user_data.user_keys[0].public_key;
		let private_key = &user.user_data.user_keys[0].private_key;
		let jwt = &user.user_data.jwt;

		let group_id = create_group(secret_token.as_str(), public_key, None, jwt).await;

		let (_, group_data_for_creator) = get_group(secret_token.as_str(), jwt, group_id.as_str(), private_key, false).await;

		let mut decrypted_group_keys = HashMap::new();

		decrypted_group_keys.insert(user.user_id.to_string(), group_data_for_creator);

		groups.push(GroupState {
			group_id,
			group_member: vec![user.user_id.to_string()],
			decrypted_group_keys,
		});
	}

	USERS_TEST_STATE
		.get_or_init(|| async move { RwLock::new(users) })
		.await;

	GROUP_TEST_STATE
		.get_or_init(|| async move { RwLock::new(groups) })
		.await;

	CONTENT_TEST_STATE
		.get_or_init(|| async move { RwLock::new(vec![]) })
		.await;
}

#[tokio::test]
async fn test_10_create_non_related_content()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let mut state = CONTENT_TEST_STATE.get().unwrap().write().await;

	let creator = &users[0];

	let url = get_url("api/v1/content".to_owned());

	//TODo do this with the sdk
	let input = sentc_crypto_common::content::CreateData {
		cat_ids: vec![],
		item: "lalala".to_string(),
	};

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(json_to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentCreateOutput = handle_server_response(&body).unwrap();

	state.push(ContentState {
		id: out.content_id,
		content: "lalala".to_string(),
	});
}

#[tokio::test]
async fn test_11_get_the_content_by_list_fetch()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let url = get_url("api/v1/content/all/0/none".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	println!("{}", body);

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, state[0].id);
	assert_eq!(out[0].item, state[0].content);
}

//insert the user after the user related content tests in a group!

//__________________________________________________________________________________________________
//clean up

#[tokio::test]
async fn zzz_clean_up()
{
	let app = APP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	for user in users.iter() {
		delete_user(secret_token, user.user_data.jwt.as_str()).await;
	}

	let customer_jwt = &CUSTOMER_TEST_STATE
		.get()
		.unwrap()
		.read()
		.await
		.user_keys
		.jwt;

	delete_app(customer_jwt, app.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

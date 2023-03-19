use std::collections::HashMap;

use reqwest::header::AUTHORIZATION;
use sentc_crypto::group::GroupKeyData;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::UserData;
use sentc_crypto_common::content::{ContentCreateOutput, ContentItemAccess, ListContentItem};
use sentc_crypto_common::group::GroupCreateOutput;
use sentc_crypto_common::{ContentId, GroupId, UserId};
use server_api_common::app::AppRegisterOutput;
use server_api_common::customer::CustomerDoneLoginOutput;
use server_core::input_helper::json_to_string;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
	add_user_by_invite,
	add_user_by_invite_as_group_as_member,
	auth_header,
	create_app,
	create_group,
	create_test_customer,
	create_test_user,
	customer_delete,
	delete_app,
	delete_user,
	get_group,
	get_group_from_group_as_member,
	get_url,
};

mod test_fn;

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerDoneLoginOutput>> = OnceCell::const_new();
static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<Vec<UserState>>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();
static CHILD_GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();
static CONNECTED_GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();

static CONTENT_TEST_STATE: OnceCell<RwLock<Vec<ContentState>>> = OnceCell::const_new();

/**
5 users:
user 1 and 2 created a group
user 3 is member of group 0
user 4 is member of the child group of group 0
user 5 is member rof a connected group to group 0
*/
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

	for i in 0..5 {
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

	let input = sentc_crypto_common::content::CreateData {
		category: None,
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

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, state[0].id);
	assert_eq!(out[0].item, state[0].content);
}

#[tokio::test]
async fn test_12_not_get_the_value_as_user_other()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;

	let creator = &users[1];
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

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_13_check_access_to_item()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let url = get_url("api/v1/content/access/item/".to_owned() + state[0].content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(out.access);
	assert_eq!(out.access_from_group, None);

	//check the other user
	let creator = &users[1];
	let url = get_url("api/v1/content/access/item/".to_owned() + state[0].content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(!out.access);
}

#[tokio::test]
async fn test_14_create_item_for_other_user()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let mut state = CONTENT_TEST_STATE.get().unwrap().write().await;

	let creator = &users[0];
	let user_1 = &users[1];

	let url = get_url("api/v1/content/".to_owned() + user_1.user_id.as_str());

	let input = sentc_crypto_common::content::CreateData {
		category: None,
		item: "lalala1".to_string(),
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
		content: "lalala1".to_string(),
	});
}

#[tokio::test]
async fn test_15_test_access_to_item_from_other_user()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let user_1 = &users[1];

	//check first for creator
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

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	//out is order by time dec
	assert_eq!(out.len(), 2);
	assert_eq!(out[0].id, state[1].id);
	assert_eq!(out[0].item, state[1].content);

	//check the 2nd page for the content
	let url = get_url("api/v1/content/all/".to_owned() + out[0].time.to_string().as_str() + "/" + out[0].id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, state[0].id);
	assert_eq!(out[0].item, state[0].content);

	//now check for the 2nd user which belongs to the content

	let url = get_url("api/v1/content/all/0/none".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user_1.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, state[1].id);
	assert_eq!(out[0].item, state[1].content);
}

#[tokio::test]
async fn test_16_not_access_by_id()
{
	/*
	This test is necessary because in the past we got db error when we send the id instead of the item
	 */

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let url = get_url("api/v1/content/access/item/".to_owned() + state[1].id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(!out.access);
}

#[tokio::test]
async fn test_16_access_check_for_the_user()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let url = get_url("api/v1/content/access/item/".to_owned() + state[1].content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(out.access);

	//check the other user
	let creator = &users[1];
	let url = get_url("api/v1/content/access/item/".to_owned() + state[1].content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(out.access);
}

#[tokio::test]
async fn test_17_delete_content_by_id()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let url = get_url("api/v1/content/id/".to_owned() + state[1].id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_18_not_access_deleted_content()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;

	//should not access the content

	let creator = &users[0];
	let url = get_url("api/v1/content/access/item/".to_owned() + state[1].content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(!out.access);

	//check the other user
	let creator = &users[1];
	let url = get_url("api/v1/content/access/item/".to_owned() + state[1].content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(!out.access);

	//should not get the item fetched in the list

	let creator = &users[0];
	let user_1 = &users[1];

	//check first for creator
	let url = get_url("api/v1/content/all/0/none".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, state[0].id);
	assert_eq!(out[0].item, state[0].content);

	//no content for the user belongs to
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user_1.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_19_delete_content_by_item()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let url = get_url("api/v1/content/item/".to_owned() + state[0].content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_20_not_access_deleted_content_by_item()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;

	//should not access the content

	let creator = &users[0];
	let url = get_url("api/v1/content/access/item/".to_owned() + state[0].content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(!out.access);

	let creator = &users[0];

	//check first for creator
	let url = get_url("api/v1/content/all/0/none".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);
}

//__________________________________________________________________________________________________
//insert the user after the user related content tests in a group!

#[tokio::test]
async fn test_21_create_groups()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;

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

	//create child group of group 0
	let jwt = &users[0].user_data.jwt;
	let keys_group_0 = groups[0]
		.decrypted_group_keys
		.get(&users[0].user_id)
		.unwrap();

	let child_group_id = create_group(
		secret_token,
		&keys_group_0[0].public_group_key,
		Some(groups[0].group_id.to_string()),
		jwt,
	)
	.await;

	let child_group_data = get_group(
		secret_token,
		jwt,
		child_group_id.as_str(),
		&keys_group_0[0].private_group_key,
		false,
	)
	.await;

	//invite the 3rd user for group 0
	add_user_by_invite(
		secret_token,
		jwt,
		groups[0].group_id.as_str(),
		keys_group_0,
		users[2].user_id.as_str(),
		&users[2].user_data.jwt,
		&users[2].user_data.user_keys[0].exported_public_key,
		&users[2].user_data.user_keys[0].private_key,
	)
	.await;

	//user 4 is member of the child of group 0
	add_user_by_invite(
		secret_token,
		jwt,
		&child_group_id,
		&child_group_data.1,
		&users[3].user_id,
		&users[3].user_data.jwt,
		&users[3].user_data.user_keys[0].exported_public_key,
		&users[3].user_data.user_keys[0].private_key,
	)
	.await;

	//create a connected group for the child group
	let group_input = sentc_crypto::group::prepare_create(&child_group_data.1[0].public_group_key).unwrap();
	let url = get_url("api/v1/group".to_owned() + "/" + &child_group_id + "/connected");

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let connected_child_out: GroupCreateOutput = handle_server_response(&body).unwrap();
	let connected_child_data = get_group_from_group_as_member(
		secret_token,
		jwt,
		&connected_child_out.group_id,
		&child_group_id,
		&child_group_data.1[0].private_group_key,
	)
	.await;

	//save the child group
	let mut decrypted_group_keys = HashMap::new();
	decrypted_group_keys.insert(users[0].user_id.to_string(), child_group_data.1);

	CHILD_GROUP_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(vec![GroupState {
					group_id: child_group_id,
					group_member: vec![],
					decrypted_group_keys,
				}])
			}
		})
		.await;

	//make a connected group from group 1
	let jwt = &users[1].user_data.jwt;
	let group_1 = &groups[1];
	let group_1_keys = &groups[1]
		.decrypted_group_keys
		.get(&users[1].user_id)
		.unwrap()[0];

	let url = get_url("api/v1/group".to_owned() + "/" + group_1.group_id.as_str() + "/connected");

	let group_input = sentc_crypto::group::prepare_create(&group_1_keys.public_group_key).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: GroupCreateOutput = handle_server_response(&body).unwrap();

	let data = get_group_from_group_as_member(
		secret_token,
		jwt,
		&out.group_id,
		&group_1.group_id,
		&group_1_keys.private_group_key,
	)
	.await;

	//add user 5 to the connected group as direct member
	add_user_by_invite_as_group_as_member(
		secret_token,
		jwt,
		&out.group_id,
		&group_1.group_id,
		&data.1,
		&users[4].user_id,
		&users[4].user_data.jwt,
		&users[4].user_data.user_keys[0].exported_public_key,
		&users[4].user_data.user_keys[0].private_key,
	)
	.await;

	let mut decrypted_group_keys = HashMap::new();
	decrypted_group_keys.insert(users[1].user_id.to_string(), data.1);

	let mut decrypted_group_keys_child = HashMap::new();
	decrypted_group_keys_child.insert(users[0].user_id.to_string(), connected_child_data.1);

	CONNECTED_GROUP_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(vec![
					GroupState {
						group_id: out.group_id,
						group_member: vec![],
						decrypted_group_keys,
					},
					GroupState {
						group_id: connected_child_out.group_id,
						group_member: vec![],
						decrypted_group_keys: decrypted_group_keys_child,
					},
				])
			}
		})
		.await;

	GROUP_TEST_STATE
		.get_or_init(|| async move { RwLock::new(groups) })
		.await;
}

#[tokio::test]
async fn test_22_create_content_for_parent_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let mut state = CONTENT_TEST_STATE.get().unwrap().write().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[2];
	let group = &groups[0];

	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id);

	let input = sentc_crypto_common::content::CreateData {
		category: None,
		item: "lalala2".to_string(),
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
		content: "lalala2".to_string(),
	});
}

#[tokio::test]
async fn test_23_access_group_content()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let group = &groups[0];
	let user = &users[0];

	let content = &state[2];

	let url = get_url("api/v1/content/access/item/".to_owned() + content.content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(out.access);

	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id + "/all/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);

	//fetch the 2nd page for groups
	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id + "/all/" + out[0].time.to_string().as_str() + "/" + &out[0].id);

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 0);

	let url = get_url("api/v1/content/all/0/none".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);
}

#[tokio::test]
async fn test_24_not_access_item_when_not_in_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;

	let content = &state[2];

	//this user is not in the group 0
	let user = &users[1];

	let url = get_url("api/v1/content/access/item/".to_owned() + content.content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(!out.access);

	//should not be on the list
	let url = get_url("api/v1/content/all/0/none".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);

	//should not access item when user is in child
	let user = &users[4];

	let url = get_url("api/v1/content/access/item/".to_owned() + content.content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(!out.access);

	let url = get_url("api/v1/content/all/0/none".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_25_create_item_in_child_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let mut state = CONTENT_TEST_STATE.get().unwrap().write().await;
	let groups = CHILD_GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[2];
	let group = &groups[0];

	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id);

	let input = sentc_crypto_common::content::CreateData {
		category: None,
		item: "lalala3".to_string(),
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
		content: "lalala3".to_string(),
	});
}

#[tokio::test]
async fn test_26_access_item_from_parent_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;
	let groups = CHILD_GROUP_TEST_STATE.get().unwrap().read().await;

	let group = &groups[0];
	let user = &users[0];

	let content = &state[3];

	let url = get_url("api/v1/content/access/item/".to_owned() + content.content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(out.access);

	//access only group content
	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id + "/all/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);

	let url = get_url("api/v1/content/all/0/none".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 2);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);
}

#[tokio::test]
async fn test_27_access_child_group_item_directly_as_direct_member()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;
	let groups = CHILD_GROUP_TEST_STATE.get().unwrap().read().await;

	let group = &groups[0];
	let user = &users[3];

	let content = &state[3];

	let url = get_url("api/v1/content/access/item/".to_owned() + content.content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(out.access);

	let url = get_url("api/v1/content/all/0/none".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);
}

#[tokio::test]
async fn test_28_create_item_in_connected_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let mut state = CONTENT_TEST_STATE.get().unwrap().write().await;
	let groups = CONNECTED_GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[4];
	let group = &groups[0];

	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id);

	let input = sentc_crypto_common::content::CreateData {
		category: None,
		item: "lalala4".to_string(),
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
		content: "lalala4".to_string(),
	});
}

#[tokio::test]
async fn test_29_access_item_from_a_connected_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;
	let groups = CONNECTED_GROUP_TEST_STATE.get().unwrap().read().await;
	let access_group = GROUP_TEST_STATE.get().unwrap().read().await;

	let group = &groups[0];
	let user = &users[1];
	let access = &access_group[1];

	let content = &state[4];

	let url = get_url("api/v1/content/access/item/".to_owned() + content.content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(out.access);
	assert_eq!(out.access_from_group.as_ref().unwrap(), &access.group_id);

	//access only group content directly from the connected group
	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id + "/all/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &access.group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);
	assert_eq!(out[0].category, None);

	//access from a connected group
	let url = get_url("api/v1/content/group/".to_owned() + &access.group_id + "/all/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);
	assert_eq!(out[0].access_from_group.as_ref().unwrap(), &access.group_id);
	assert_eq!(out[0].category, None);

	//access global
	let url = get_url("api/v1/content/all/0/none".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);
	assert_eq!(out[0].access_from_group.as_ref().unwrap(), &access.group_id);
	assert_eq!(out[0].category, None);
}

#[tokio::test]
async fn test_30_create_content_in_a_connected_group_which_is_connected_to_a_child()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let mut state = CONTENT_TEST_STATE.get().unwrap().write().await;
	let groups = CONNECTED_GROUP_TEST_STATE.get().unwrap().read().await;
	let access_group = CHILD_GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[2];
	let group = &groups[1];
	let access = &access_group[0];

	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id);

	let input = sentc_crypto_common::content::CreateData {
		category: None,
		item: "lalala5".to_string(),
	};

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &access.group_id)
		.body(json_to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentCreateOutput = handle_server_response(&body).unwrap();

	state.push(ContentState {
		id: out.content_id,
		content: "lalala5".to_string(),
	});
}

#[tokio::test]
async fn test_31_access_item_from_a_connected_group_and_parent()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;
	let groups = CONNECTED_GROUP_TEST_STATE.get().unwrap().read().await;
	let access_group = CHILD_GROUP_TEST_STATE.get().unwrap().read().await;

	let group = &groups[1];
	let user = &users[0];
	let access = &access_group[0];

	let content = &state[5];

	let url = get_url("api/v1/content/access/item/".to_owned() + content.content.as_str());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentItemAccess = handle_server_response(&body).unwrap();

	assert!(out.access);
	assert_eq!(out.access_from_group.as_ref().unwrap(), &access.group_id);

	//access only group content directly from the connected group
	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id + "/all/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &access.group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);
	assert_eq!(out[0].category, None);

	//access from a connected group
	let url = get_url("api/v1/content/group/".to_owned() + &access.group_id + "/all/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 2);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);
	assert_eq!(out[0].access_from_group.as_ref().unwrap(), &access.group_id);
	assert_eq!(out[0].category, None);

	let url = get_url("api/v1/content/all/0/none".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 3);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);
	assert_eq!(out[0].access_from_group.as_ref().unwrap(), &access.group_id)
}

//__________________________________________________________________________________________________
//category test

#[tokio::test]
async fn test_32_create_non_related_content_with_category()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let mut state = CONTENT_TEST_STATE.get().unwrap().write().await;

	let creator = &users[0];

	let url = get_url("api/v1/content".to_owned());

	let input = sentc_crypto_common::content::CreateData {
		category: Some("abc".to_string()),
		item: "lalala_cat".to_string(),
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
		content: "lalala_cat".to_string(),
	});
}

#[tokio::test]
async fn test_33_fetch_content_with_cat()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let url = get_url("api/v1/content/".to_owned() + "abc" + "/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, state[6].id);
	assert_eq!(out[0].item, state[6].content);
	assert_eq!(out[0].category, Some("abc".to_string()));

	//2nd page fetch
	let url = get_url("api/v1/content/".to_owned() + "abc" + "/" + out[0].time.to_string().as_str() + "/" + &out[0].id);

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);

	//fetch with wrong cat
	let url = get_url("api/v1/content/".to_owned() + "abc1" + "/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_34_create_group_content_with_cat()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let mut state = CONTENT_TEST_STATE.get().unwrap().write().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[2];
	let group = &groups[0];

	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id);

	let input = sentc_crypto_common::content::CreateData {
		category: Some("abc".to_string()),
		item: "lalala_cat1".to_string(),
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
		content: "lalala_cat1".to_string(),
	});
}

#[tokio::test]
async fn test_35_access_group_content_with_cat()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let group = &groups[0];
	let user = &users[0];

	let content = &state[7];

	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id + "/" + "abc" + "/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);
	assert_eq!(out[0].category, Some("abc".to_string()));

	//normal access too
	let url = get_url("api/v1/content/".to_owned() + "abc" + "/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 2);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].item, content.content);
	assert_eq!(out[0].category, Some("abc".to_string()));
}

#[tokio::test]
async fn test_36_create_content_in_a_child_from_a_connected_group_with_cat()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let mut state = CONTENT_TEST_STATE.get().unwrap().write().await;
	let groups = CONNECTED_GROUP_TEST_STATE.get().unwrap().read().await;
	let access_group = CHILD_GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[2];
	let group = &groups[1];
	let access = &access_group[0];

	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id);

	let input = sentc_crypto_common::content::CreateData {
		category: Some("abc".to_string()),
		item: "lalala_cat2".to_string(),
	};

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &access.group_id)
		.body(json_to_string(&input).unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: ContentCreateOutput = handle_server_response(&body).unwrap();

	state.push(ContentState {
		id: out.content_id,
		content: "lalala_cat2".to_string(),
	});
}

#[tokio::test]
async fn test_37_access_content_in_connected_group_with_cat()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let state = CONTENT_TEST_STATE.get().unwrap().read().await;
	let groups = CONNECTED_GROUP_TEST_STATE.get().unwrap().read().await;
	let access_group = CHILD_GROUP_TEST_STATE.get().unwrap().read().await;

	let group = &groups[1];
	let user = &users[0];
	let access = &access_group[0];

	let content = &state[8];

	let url = get_url("api/v1/content/group/".to_owned() + &group.group_id + "/" + "abc" + "/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url.clone())
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &access.group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListContentItem> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, content.id);
	assert_eq!(out[0].belongs_to_group.as_ref().unwrap(), &group.group_id);
	assert_eq!(out[0].category, Some("abc".to_string()));
}

//__________________________________________________________________________________________________
//clean up

#[tokio::test]
async fn zzz_clean_up()
{
	let app = APP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	for user in users.iter() {
		delete_user(secret_token, &user.user_data.user_id).await;
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

use std::collections::HashMap;

use reqwest::header::AUTHORIZATION;
use sentc_crypto::group::GroupKeyData;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::util::HmacKeyFormat;
use sentc_crypto::UserData;
use sentc_crypto_common::content_searchable::{ListSearchItem, SearchCreateData};
use sentc_crypto_common::group::GroupCreateOutput;
use sentc_crypto_common::{GroupId, UserId};
use server_api::util::api_res::ApiErrorCodes;
use server_api_common::app::AppRegisterOutput;
use server_api_common::customer::CustomerDoneLoginOutput;
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
	decrypt_group_hmac_keys,
	delete_app,
	delete_user,
	get_group,
	get_group_from_group_as_member,
	get_server_error_from_normal_res,
	get_url,
};

mod test_fn;

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerDoneLoginOutput>> = OnceCell::const_new();
static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<Vec<UserState>>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();

//the item refs
static SEARCH_ITEM: OnceCell<RwLock<Vec<String>>> = OnceCell::const_new();

/**
Test group only for now.

Test access from:
- group
- parent group (access the child from parent)
- group as member (assess from connected group)
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
	pub hmac_keys: Vec<HmacKeyFormat>,
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

	for i in 0..4 {
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

	//______________________________________________________________________________________________
	//create normal group / user 1 creator, user 2 direct member
	let creator = &users[0];
	let creator_jwt = &creator.user_data.jwt;
	let creator_public_key = &creator.user_data.user_keys[0].public_key;
	let creator_private_key = &creator.user_data.user_keys[0].private_key;

	let group_id = create_group(secret_token.as_str(), creator_public_key, None, creator_jwt).await;

	let (data, group_data_for_creator) = get_group(
		secret_token.as_str(),
		creator_jwt,
		group_id.as_str(),
		creator_private_key,
		false,
	)
	.await;

	let hmac_keys = decrypt_group_hmac_keys(&group_data_for_creator[0].group_key, &data.hmac_keys);

	let mut decrypted_group_keys = HashMap::new();

	decrypted_group_keys.insert(creator.user_id.to_string(), group_data_for_creator);

	let group_0 = GroupState {
		group_id: group_id.clone(),
		group_member: vec![creator.user_id.to_string()],
		decrypted_group_keys,
		hmac_keys,
	};

	let member = &users[1];

	let keys_group_0 = group_0.decrypted_group_keys.get(&creator.user_id).unwrap();

	add_user_by_invite(
		secret_token_str,
		creator_jwt,
		group_0.group_id.as_str(),
		keys_group_0,
		member.user_id.as_str(),
		&member.user_data.jwt,
		&member.user_data.user_keys[0].exported_public_key,
		&member.user_data.user_keys[0].private_key,
	)
	.await;

	//______________________________________________________________________________________________
	//create child from the normal group / user 3 direct member
	let child_group_id = create_group(
		secret_token_str,
		&keys_group_0[0].public_group_key,
		Some(group_0.group_id.to_string()),
		creator_jwt,
	)
	.await;

	let (data, group_data_for_creator) = get_group(
		secret_token_str,
		creator_jwt,
		child_group_id.as_str(),
		&keys_group_0[0].private_group_key,
		false,
	)
	.await;

	let hmac_keys = decrypt_group_hmac_keys(&group_data_for_creator[0].group_key, &data.hmac_keys);

	let mut decrypted_group_keys = HashMap::new();

	decrypted_group_keys.insert(creator.user_id.to_string(), group_data_for_creator);

	let group_1 = GroupState {
		group_id: child_group_id.clone(),
		group_member: vec![creator.user_id.to_string()],
		decrypted_group_keys,
		hmac_keys,
	};

	let member = &users[2];

	let keys_group_1 = group_1.decrypted_group_keys.get(&creator.user_id).unwrap();

	add_user_by_invite(
		secret_token_str,
		creator_jwt,
		group_1.group_id.as_str(),
		keys_group_1,
		member.user_id.as_str(),
		&member.user_data.jwt,
		&member.user_data.user_keys[0].exported_public_key,
		&member.user_data.user_keys[0].private_key,
	)
	.await;

	//______________________________________________________________________________________________
	//create connected group to normal group / user 4 direct member
	let group_input = sentc_crypto::group::prepare_create(&keys_group_0[0].public_group_key).unwrap();
	let url = get_url("api/v1/group".to_owned() + "/" + &group_id + "/connected");

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator_jwt))
		.header("x-sentc-app-token", secret_token_str)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let connected: GroupCreateOutput = handle_server_response(&body).unwrap();
	let (connected_data, connected_keys) = get_group_from_group_as_member(
		secret_token_str,
		creator_jwt,
		&connected.group_id,
		&group_id,
		&keys_group_0[0].private_group_key,
	)
	.await;

	let hmac_keys = decrypt_group_hmac_keys(&connected_keys[0].group_key, &connected_data.hmac_keys);

	let mut decrypted_group_keys = HashMap::new();
	decrypted_group_keys.insert(creator.user_id.to_string(), connected_keys);

	let group_3 = GroupState {
		group_id: connected.group_id.clone(),
		group_member: vec![creator.user_id.to_string()],
		decrypted_group_keys,
		hmac_keys,
	};

	let member = &users[3];

	add_user_by_invite_as_group_as_member(
		secret_token_str,
		creator_jwt,
		&connected.group_id,
		&group_0.group_id,
		keys_group_0,
		&member.user_id,
		&member.user_data.jwt,
		&member.user_data.user_keys[0].exported_public_key,
		&member.user_data.user_keys[0].private_key,
	)
	.await;
	//______________________________________________________________________________________________

	USERS_TEST_STATE
		.get_or_init(|| async move { RwLock::new(users) })
		.await;

	SEARCH_ITEM
		.get_or_init(|| async move { RwLock::new(vec![]) })
		.await;

	GROUP_TEST_STATE
		.get_or_init(|| async move { RwLock::new(vec![group_0, group_1, group_3]) })
		.await;
}

//__________________________________________________________________________________________________

#[tokio::test]
async fn test_10_create_full_hash()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	//create an searchable item in the main group
	let creator = &users[0];
	let group = &groups[0];

	let data = "123*+^êéèüöß@€&$ 👍 🚀 😎";

	let input = sentc_crypto::crypto_searchable::create_searchable(&group.hmac_keys[0], "ref", None, data, true, None).unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_11_search_full_hash()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let group = &groups[0];

	let data = "123*+^êéèüöß@€&$ 👍 🚀 😎";

	//should not find ref when data is not full
	let search_str = sentc_crypto::crypto_searchable::search(&group.hmac_keys[0], "123").unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id + "/all/0/none?search=" + &search_str);

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<ListSearchItem> = handle_server_response(&body).unwrap();

	assert_eq!(list.len(), 0);

	//should find the ref with full data

	let search_str = sentc_crypto::crypto_searchable::search(&group.hmac_keys[0], data).unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id + "/all/0/none?search=" + &search_str);

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<ListSearchItem> = handle_server_response(&body).unwrap();

	assert_eq!(list.len(), 1);
	assert_eq!(list[0].item_ref, "ref".to_string());
}

#[tokio::test]
async fn test_12_create_hash()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	//create an searchable item in the main group
	let creator = &users[0];
	let group = &groups[0];

	let data = "123*+^êéèüöß@€&$ 👍 🚀 😎";

	let input = sentc_crypto::crypto_searchable::create_searchable(&group.hmac_keys[0], "ref1", None, data, false, None).unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_13_search_hash_with_parts()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let public_token = &APP_TEST_STATE.get().unwrap().read().await.public_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let group = &groups[0];

	let data = "123*+^êéèüöß@€&$ 👍 🚀 😎";

	//should find ref when data is not full
	let search_str = sentc_crypto::crypto_searchable::search(&group.hmac_keys[0], "123").unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id + "/all/0/none?search=" + &search_str);

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<ListSearchItem> = handle_server_response(&body).unwrap();

	assert_eq!(list.len(), 1);
	assert_eq!(list[0].item_ref, "ref1".to_string());

	//should find the ref with full data

	let search_str = sentc_crypto::crypto_searchable::search(&group.hmac_keys[0], data).unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id + "/all/0/none?search=" + &search_str);

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", public_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<ListSearchItem> = handle_server_response(&body).unwrap();

	//2 because the full hash and the part hash
	assert_eq!(list.len(), 2);
	assert_eq!(list[0].item_ref, "ref1".to_string());
	assert_eq!(list[1].item_ref, "ref".to_string());
}

#[tokio::test]
async fn test_14_search_hash_pagination()
{
	let public_token = &APP_TEST_STATE.get().unwrap().read().await.public_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let group = &groups[0];

	let data = "123*+^êéèüöß@€&$ 👍 🚀 😎";

	let search_str = sentc_crypto::crypto_searchable::search(&group.hmac_keys[0], data).unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id + "/all/0/none?search=" + &search_str);

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", public_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<ListSearchItem> = handle_server_response(&body).unwrap();

	//2 because the full hash and the part hash
	assert_eq!(list.len(), 2);
	assert_eq!(list[0].item_ref, "ref1".to_string());
	assert_eq!(list[1].item_ref, "ref".to_string());

	//2nd page
	let url = get_url(format!(
		"api/v1/search/group/{}/all/{}/{}?search={}",
		&group.group_id, list[0].time, list[0].id, &search_str
	));

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", public_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<ListSearchItem> = handle_server_response(&body).unwrap();

	assert_eq!(list.len(), 1);
	assert_eq!(list[0].item_ref, "ref".to_string());
}

#[tokio::test]
async fn test_15_create_hash_in_child_group_and_search_from_parent()
{
	//use here the hmac key of the child

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let public_token = &APP_TEST_STATE.get().unwrap().read().await.public_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[2];
	let group = &groups[1];

	let data = "123*+^êéèüöß@€&$ 👍 🚀 😎";

	let input = sentc_crypto::crypto_searchable::create_searchable(&group.hmac_keys[0], "ref2", None, data, false, None).unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//now access it from the parent group
	let creator = &users[1]; //direct member of the parent group (not the creator)

	let search_str = sentc_crypto::crypto_searchable::search(&group.hmac_keys[0], "123").unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id + "/all/0/none?search=" + &search_str);

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", public_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<ListSearchItem> = handle_server_response(&body).unwrap();

	//should not find the other hashes because these are for the parent group
	assert_eq!(list.len(), 1);
	assert_eq!(list[0].item_ref, "ref2".to_string());
}

#[tokio::test]
async fn test_16_create_hash_in_connected_group_and_search()
{
	//use here the hmac key of the child

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[3];
	let group = &groups[2];

	let data = "123*+^êéèüöß@€&$ 👍 🚀 😎";

	let input = sentc_crypto::crypto_searchable::create_searchable(&group.hmac_keys[0], "ref3", None, data, false, None).unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//now access it from the connected group group

	let creator = &users[1]; //direct member of the connected group (not the creator)

	let search_str = sentc_crypto::crypto_searchable::search(&group.hmac_keys[0], "123").unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id + "/all/0/none?search=" + &search_str);

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &groups[0].group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<ListSearchItem> = handle_server_response(&body).unwrap();

	//should not find the other hashes because these are for the parent group
	assert_eq!(list.len(), 1);
	assert_eq!(list[0].item_ref, "ref3".to_string());
}

#[tokio::test]
async fn test_17_not_create_without_ref()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let group = &groups[0];

	let data = "123*+^êéèüöß@€&$ 👍 🚀 😎";

	let input = sentc_crypto::crypto_searchable::create_searchable(&group.hmac_keys[0], "", None, data, false, None).unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(
		server_err,
		ApiErrorCodes::ContentSearchableItemRefNotSet.get_int_code()
	);
}

#[tokio::test]
async fn test_18_not_create_with_too_long_ref()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let group = &groups[0];

	let data = "123*+^êéèüöß@€&$ 👍 🚀 😎";

	let mut item_ref = "".to_string();

	for i in 0..51 {
		item_ref += i.to_string().as_str();
	}

	let input = sentc_crypto::crypto_searchable::create_searchable(&group.hmac_keys[0], &item_ref, None, data, false, None).unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(
		server_err,
		ApiErrorCodes::ContentSearchableItemRefTooBig.get_int_code()
	);
}

#[tokio::test]
async fn test_19_not_create_with_empty_hashes()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let group = &groups[0];

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(
			serde_json::to_string(&SearchCreateData {
				category: None,
				item_ref: "hello".to_string(),
				hashes: vec![],
				alg: "".to_string(),
				key_id: "".to_string(),
			})
			.unwrap(),
		)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, ApiErrorCodes::ContentSearchableNoHashes.get_int_code());
}

#[tokio::test]
async fn test_20_not_create_with_too_many_hashes()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let group = &groups[0];

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id);

	let mut hashes = Vec::with_capacity(201);

	for i in 0..201 {
		hashes.push(i.to_string());
	}

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(
			serde_json::to_string(&SearchCreateData {
				category: None,
				item_ref: "hello".to_string(),
				hashes,
				alg: "".to_string(),
				key_id: "".to_string(),
			})
			.unwrap(),
		)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(
		server_err,
		ApiErrorCodes::ContentSearchableTooManyHashes.get_int_code()
	);
}

#[tokio::test]
async fn test_21_delete_hash()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = &users[0];
	let group = &groups[0];

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id + "/ref");

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

	//should not get the deleted item
	let data = "123*+^êéèüöß@€&$ 👍 🚀 😎";

	let search_str = sentc_crypto::crypto_searchable::search(&group.hmac_keys[0], data).unwrap();

	let url = get_url("api/v1/search/group/".to_owned() + &group.group_id + "/all/0/none?search=" + &search_str);

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<ListSearchItem> = handle_server_response(&body).unwrap();

	//only one item left
	assert_eq!(list.len(), 1);
	assert_eq!(list[0].item_ref, "ref1");
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

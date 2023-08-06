use std::collections::HashMap;

use sentc_crypto::entities::group::GroupKeyData;
use sentc_crypto::entities::keys::HmacKeyFormatInt;
use sentc_crypto::entities::user::UserDataInt;
use sentc_crypto_common::{GroupId, UserId};
use server_dashboard_common::app::AppRegisterOutput;
use server_dashboard_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
	create_app,
	create_group,
	create_test_customer,
	create_test_user,
	customer_delete,
	decrypt_group_hmac_keys,
	delete_app,
	delete_user,
	get_group,
};

mod test_fn;

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerDoneLoginOutput>> = OnceCell::const_new();
static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<UserState>> = OnceCell::const_new();

pub struct UserState
{
	pub username: String,
	pub pw: String,
	pub user_id: UserId,
	pub user_data: UserDataInt,
}

pub struct GroupState
{
	pub group_id: GroupId,
	pub group_member: Vec<UserId>,
	pub decrypted_group_keys: HashMap<UserId, Vec<GroupKeyData>>,
	pub searchable_keys: Vec<HmacKeyFormatInt>,
}

const STR: &str = "123*+^êéèüöß@€&$ 👍 🚀 😎";

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

	let user_pw = "12345";

	let secret_token_str = secret_token.as_str();
	let public_token_str = public_token.as_str();

	let username = "hi4";

	let (user_id, key_data) = create_test_user(secret_token_str, public_token_str, username, user_pw).await;

	let user = UserState {
		username: username.to_string(),
		pw: user_pw.to_string(),
		user_id,
		user_data: key_data,
	};

	//create normal group / user 1 creator, user 2 direct member
	let creator = &user;
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

	let searchable_keys = decrypt_group_hmac_keys(&group_data_for_creator[0].group_key, data.hmac_keys);

	let mut decrypted_group_keys = HashMap::new();

	decrypted_group_keys.insert(creator.user_id.to_string(), group_data_for_creator);

	let group_0 = GroupState {
		group_id: group_id.clone(),
		group_member: vec![creator.user_id.to_string()],
		decrypted_group_keys,
		searchable_keys,
	};

	//2nd group to test encrypt with other key
	let group_id = create_group(secret_token.as_str(), creator_public_key, None, creator_jwt).await;

	let (data, group_data_for_creator) = get_group(
		secret_token.as_str(),
		creator_jwt,
		group_id.as_str(),
		creator_private_key,
		false,
	)
	.await;

	let searchable_keys = decrypt_group_hmac_keys(&group_data_for_creator[0].group_key, data.hmac_keys);

	let mut decrypted_group_keys = HashMap::new();

	decrypted_group_keys.insert(creator.user_id.to_string(), group_data_for_creator);

	let group_1 = GroupState {
		group_id: group_id.clone(),
		group_member: vec![creator.user_id.to_string()],
		decrypted_group_keys,
		searchable_keys,
	};

	USERS_TEST_STATE
		.get_or_init(|| async move { RwLock::new(user) })
		.await;

	GROUP_TEST_STATE
		.get_or_init(|| async move { RwLock::new(vec![group_0, group_1]) })
		.await;
}

#[tokio::test]
async fn test_10_create_searchable_full()
{
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let group = &groups[0];

	let out = sentc_crypto::crypto_searchable::create_searchable_raw(&group.searchable_keys[0], STR, true, None).unwrap();

	assert_eq!(out.len(), 1);
}

#[tokio::test]
async fn test_11_create_searchable()
{
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let group = &groups[0];

	let out = sentc_crypto::crypto_searchable::create_searchable_raw(&group.searchable_keys[0], STR, false, None).unwrap();

	assert_eq!(out.len(), 39);
}

#[tokio::test]
async fn test_12_search_item_full()
{
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let group = &groups[0];
	let key = &group.searchable_keys[0];

	let out = sentc_crypto::crypto_searchable::create_searchable_raw(key, STR, true, None).unwrap();

	assert_eq!(out.len(), 1);

	let search_str = sentc_crypto::crypto_searchable::search(key, "123").unwrap();

	//should not contains only a part of the word because we used full
	assert!(!out.contains(&search_str));

	//but should contains the full word
	let search_str = sentc_crypto::crypto_searchable::search(key, STR).unwrap();

	assert!(out.contains(&search_str));
}

#[tokio::test]
async fn test_13_search_item()
{
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let group = &groups[0];
	let key = &group.searchable_keys[0];

	let out = sentc_crypto::crypto_searchable::create_searchable_raw(key, STR, false, None).unwrap();
	assert_eq!(out.len(), 39);

	//now get the output of the prepare search
	let search_str = sentc_crypto::crypto_searchable::search(key, "123").unwrap();
	assert!(out.contains(&search_str));
}

#[tokio::test]
async fn test_14_not_search_item_with_different_keys()
{
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let group = &groups[0];
	let key = &group.searchable_keys[0];

	let group2 = &groups[1];
	let key2 = &group2.searchable_keys[0];

	let out = sentc_crypto::crypto_searchable::create_searchable_raw(key, STR, false, None).unwrap();

	let search_str = sentc_crypto::crypto_searchable::search(key, "123").unwrap();

	let search_str2 = sentc_crypto::crypto_searchable::search(key2, "123").unwrap();

	assert_ne!(search_str, search_str2);

	assert!(out.contains(&search_str));
	assert!(!out.contains(&search_str2));
}

//__________________________________________________________________________________________________
//clean up

#[tokio::test]
async fn zzz_clean_up()
{
	let app = APP_TEST_STATE.get().unwrap().read().await;

	let user = USERS_TEST_STATE.get().unwrap().read().await;

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	delete_user(secret_token, &user.user_data.user_id).await;

	let customer_jwt = &CUSTOMER_TEST_STATE.get().unwrap().read().await.verify.jwt;

	delete_app(customer_jwt, app.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

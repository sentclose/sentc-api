use std::collections::HashMap;

use sentc_crypto::sdk_core::cryptomat::SortableKey as CoreSort;
use sentc_crypto::sdk_utils::cryptomat::SortableKeyWrapper;
use sentc_crypto::std_keys::util::SortableKey;
use sentc_crypto::{StdGroupKeyData, StdUserDataInt};
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
	decrypt_group_sortable_keys,
	delete_app,
	delete_user,
	get_group,
};

mod test_fn;

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerDoneLoginOutput>> = OnceCell::const_new();
static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<UserState>> = OnceCell::const_new();
static NUMBERS: OnceCell<RwLock<Vec<u64>>> = OnceCell::const_new();

pub struct UserState
{
	pub username: String,
	pub pw: String,
	pub user_id: UserId,
	pub user_data: StdUserDataInt,
}

pub struct GroupState
{
	pub group_id: GroupId,
	pub group_member: Vec<UserId>,
	pub decrypted_group_keys: HashMap<UserId, Vec<StdGroupKeyData>>,
	pub sortable_keys: Vec<SortableKey>,
}

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

	let sortable_keys = decrypt_group_sortable_keys(&group_data_for_creator[0].group_key, data.sortable_keys);

	let mut decrypted_group_keys = HashMap::new();

	decrypted_group_keys.insert(creator.user_id.to_string(), group_data_for_creator);

	let group_0 = GroupState {
		group_id: group_id.clone(),
		group_member: vec![creator.user_id.to_string()],
		decrypted_group_keys,
		sortable_keys,
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

	let sortable_keys = decrypt_group_sortable_keys(&group_data_for_creator[0].group_key, data.sortable_keys);

	let mut decrypted_group_keys = HashMap::new();

	decrypted_group_keys.insert(creator.user_id.to_string(), group_data_for_creator);

	let group_1 = GroupState {
		group_id: group_id.clone(),
		group_member: vec![creator.user_id.to_string()],
		decrypted_group_keys,
		sortable_keys,
	};

	USERS_TEST_STATE
		.get_or_init(|| async move { RwLock::new(user) })
		.await;

	GROUP_TEST_STATE
		.get_or_init(|| async move { RwLock::new(vec![group_0, group_1]) })
		.await;

	NUMBERS.get_or_init(|| async { RwLock::new(vec![]) }).await;
}

#[tokio::test]
async fn test_10_encrypt_number()
{
	let key = &GROUP_TEST_STATE.get().unwrap().read().await[0].sortable_keys[0];

	let a = key.encrypt_sortable(262).unwrap();
	let b = key.encrypt_sortable(263).unwrap();
	let c = key.encrypt_sortable(65321).unwrap();

	assert!(a < b);
	assert!(b < c);

	let mut n = NUMBERS.get().unwrap().write().await;
	n.push(a);
	n.push(b);
	n.push(c);
}

#[tokio::test]
async fn test_11_encrypt_number_with_other_group()
{
	let key = &GROUP_TEST_STATE.get().unwrap().read().await[1].sortable_keys[0];
	let n = NUMBERS.get().unwrap().read().await;

	let a = key.encrypt_sortable(262).unwrap();
	let b = key.encrypt_sortable(263).unwrap();
	let c = key.encrypt_sortable(65321).unwrap();

	assert!(a < b);
	assert!(b < c);

	assert_ne!(a, n[0]);
	assert_ne!(b, n[1]);
	assert_ne!(c, n[2]);
}

#[tokio::test]
async fn test_12_encrypt_string()
{
	let key = &GROUP_TEST_STATE.get().unwrap().read().await[1].sortable_keys[0];

	const STR_VALUES: [&str; 10] = ["a", "az", "azzz", "b", "ba", "baaa", "o", "oe", "z", "zaaa"];
	let mut encrypted_vars = vec![];

	for v in STR_VALUES {
		encrypted_vars.push(key.encrypt_raw_string(v, None).unwrap())
	}

	//check
	let mut past_item = 0;

	for item in encrypted_vars {
		assert!(past_item < item);

		past_item = item;
	}
}

#[tokio::test]
async fn test_20_with_generated_key()
{
	const KEY: &str = r#"{"Ope16":{"key":"5kGPKgLQKmuZeOWQyJ7vOg==","key_id":"1876b629-5795-471f-9704-0cac52eaf9a1"}}"#;

	let key: SortableKey = KEY.parse().unwrap();

	let a = key.encrypt_sortable(262).unwrap();
	let b = key.encrypt_sortable(263).unwrap();
	let c = key.encrypt_sortable(65321).unwrap();

	assert!(a < b);
	assert!(b < c);

	assert_eq!(a, 17455249);
	assert_eq!(b, 17488544);
	assert_eq!(c, 4280794268);
}

//__________________________________________________________________________________________________
//clean up

#[tokio::test]
async fn zzz_clean_up()
{
	let app = APP_TEST_STATE.get().unwrap().read().await;

	let user = USERS_TEST_STATE.get().unwrap().read().await;

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	delete_user(secret_token, user.username.clone()).await;

	let customer_jwt = &CUSTOMER_TEST_STATE.get().unwrap().read().await.verify.jwt;

	delete_app(customer_jwt, app.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

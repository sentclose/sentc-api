use std::collections::HashMap;

use sentc_crypto::group::GroupKeyData;
use sentc_crypto::UserData;
use sentc_crypto_common::{GroupId, UserId};
use server_api_common::app::AppRegisterOutput;
use server_api_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
	add_group_by_invite,
	create_app,
	create_group,
	create_test_customer,
	create_test_user,
	customer_delete,
	delete_app,
	delete_user,
	get_group,
};

/**
# Group 2nd test file

## Test here Group member in another group.

Use cases:

1. user is direct group member (tested in group tests 1)
2. user is member of a parent group (tested in group test 1)
2.1 user is member of a parent group of a group which is parent of the selected group (parent -> parent -> child to access)

3. user is direct member of a group which is member of the selected group
4. user is member of a parent of a group which is member of the selected group
5. user is direct member of a group which is member of a parent group of the selected group
6. user is member of a parent group which child is member of a parent of the selected group

## Test also

- create from group
- invite / join
- kick group from other group
- leave as group from other group
- key rotation
*/
mod test_fn;

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerDoneLoginOutput>> = OnceCell::const_new();
static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<Vec<UserState>>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();
static CHILD_GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();

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

	for i in 0..7 {
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
}

/**
create groups:
1. the group, to test direct access to a connected group
2. the group, another group with direct access to a connected group
3. the group to connect
4. a parent group and one child group which got access to a connected group
5. a group to connect (but from user and connect it later), with a child (connect group 1 and 2)
6. later a group to connect but created from group 1

At the end connect a child from one group to a parent for another group and
check if user from one parent got access at the other parent
 */
#[tokio::test]
async fn test_01_create_groups()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;

	let n = 5;
	let mut groups = Vec::with_capacity(n);

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

	//create now the children:
	// one for group 4 to test access from parent to connected group (group 3)
	//one for group 5 and connect later group 1 and 2 to test access via connected parent group

	let mut children = Vec::with_capacity(2);

	let groups_and_users = vec![(3, 3), (4, 4)];

	for (group_i, user_i) in groups_and_users {
		let group = &groups[group_i];
		let user = &users[user_i]; //use always another user

		let keys = &group
			.decrypted_group_keys
			.get(user.user_id.as_str())
			.unwrap()[0];
		let public_key = &keys.public_group_key;
		let private_key = &keys.private_group_key;
		let jwt = &user.user_data.jwt;

		let group_id = create_group(
			secret_token.as_str(),
			public_key,
			Some(group.group_id.to_string()),
			jwt,
		)
		.await;

		let (_, child_keys) = get_group(secret_token.as_str(), jwt, group_id.as_str(), private_key, false).await;

		let mut decrypted_group_keys = HashMap::new();

		decrypted_group_keys.insert(user.user_id.to_string(), child_keys);

		children.push(GroupState {
			group_id,
			group_member: vec![],
			decrypted_group_keys,
		});
	}

	GROUP_TEST_STATE
		.get_or_init(|| async move { RwLock::new(groups) })
		.await;

	CHILD_GROUP_TEST_STATE
		.get_or_init(|| async move { RwLock::new(children) })
		.await;
}

//__________________________________________________________________________________________________

//TODO test key rotation when a group or a user group got multiple keys, which key is sused or all keys are used

#[tokio::test]
async fn test_10_connect_group_to_other_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let group_1 = &groups[0];
	let creator_group_1 = &users[0];
	let group_1_private_key = &group_1
		.decrypted_group_keys
		.get(&group_1.group_member[0])
		.unwrap()[0]
		.private_group_key;

	let group_2 = &groups[1];
	let creator_group_2 = &users[1];

	let group_to_connect = &groups[2];
	let creator_group_to_connect = &users[2];

	let user_keys = group_to_connect
		.decrypted_group_keys
		.get(&group_to_connect.group_member[0])
		.unwrap();

	add_group_by_invite(
		secret_token,
		&creator_group_to_connect.user_data.jwt,
		&group_to_connect.group_id,
		&user_keys,
		&group_1.group_id,
		&creator_group_1.user_data.jwt,
		&group_1_private_key,
	)
	.await;
}

#[tokio::test]
async fn test_11_connect_group_to_other_group_with_a_child_group()
{
	//
}

#[tokio::test]
async fn test_12_connect_child_group_to_other_group()
{
	//
}

#[tokio::test]
async fn test_13_connect_group_by_creating_from_other_group()
{
	//
}

//TODO join req, invite req, reject join, reject invite, accept join, accept invite
//TODO delete group from other group

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

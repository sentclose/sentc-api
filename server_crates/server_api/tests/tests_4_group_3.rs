use std::collections::HashMap;

use sentc_crypto::group::{GroupKeyData, GroupOutData};
use sentc_crypto::UserData;
use sentc_crypto_common::group::GroupKeysForNewMemberServerInput;
use sentc_crypto_common::{GroupId, ServerOutput, UserId};
use server_api_common::app::AppRegisterOutput;
use server_api_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
	create_app,
	create_group,
	create_test_customer,
	create_test_user,
	customer_delete,
	delete_app,
	delete_user,
	get_group,
	get_group_from_group_as_member,
};

mod test_fn;

/**
This tests are all about child group of a child group of a parent and so on
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
	pub parent_id: Option<GroupId>,
	pub group_data: GroupOutData,
	pub decrypted_group_keys: HashMap<UserId, Vec<GroupKeyData>>,
}

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerDoneLoginOutput>> = OnceCell::const_new();
static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<Vec<UserState>>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();

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
}

/**
create groups:

1. parent group
2. child of 1
3. child of 2
*/
#[tokio::test]
async fn test_01_create_groups()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;

	//1. group
	let user = &users[0];
	let public_key = &user.user_data.user_keys[0].public_key;
	let private_key = &user.user_data.user_keys[0].private_key;
	let jwt = &user.user_data.jwt;

	let group_id_1 = create_group(secret_token.as_str(), public_key, None, jwt).await;

	let (data_1, group_1_data_for_creator) = get_group(secret_token.as_str(), jwt, &group_id_1, private_key, false).await;

	//2. group, child of 1
	let public_key = &group_1_data_for_creator[0].public_group_key;
	let private_key = &group_1_data_for_creator[0].private_group_key;
	let jwt = &user.user_data.jwt;

	let group_id_2 = create_group(secret_token.as_str(), public_key, Some(group_id_1.clone()), jwt).await;

	let (data_2, group_2_data_for_creator) = get_group(secret_token.as_str(), jwt, &group_id_2, private_key, false).await;

	//3. group, child of 2
	let public_key = &group_2_data_for_creator[0].public_group_key;
	let private_key = &group_2_data_for_creator[0].private_group_key;
	let jwt = &user.user_data.jwt;

	let group_id_3 = create_group(secret_token.as_str(), public_key, Some(group_id_2.clone()), jwt).await;

	let (data_3, group_3_data_for_creator) = get_group(secret_token.as_str(), jwt, &group_id_3, private_key, false).await;

	//4. group, connect to the 1. group (via service)
	let public_key = &user.user_data.user_keys[0].public_key;
	let private_key = &user.user_data.user_keys[0].private_key;

	let group_id_4 = create_group(secret_token.as_str(), public_key, None, jwt).await;

	let (data_4, group_4_data_for_creator) = get_group(secret_token.as_str(), jwt, &group_id_4, private_key, false).await;

	//save the data

	GROUP_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(vec![
					GroupState {
						group_id: group_id_1.clone(),
						parent_id: None,
						group_data: data_1,
						decrypted_group_keys: HashMap::from([(user.user_id.to_string(), group_1_data_for_creator)]),
					},
					GroupState {
						group_id: group_id_2.clone(),
						parent_id: Some(group_id_1),
						group_data: data_2,
						decrypted_group_keys: HashMap::from([(user.user_id.to_string(), group_2_data_for_creator)]),
					},
					GroupState {
						group_id: group_id_3,
						parent_id: Some(group_id_2),
						group_data: data_3,
						decrypted_group_keys: HashMap::from([(user.user_id.to_string(), group_3_data_for_creator)]),
					},
					GroupState {
						group_id: group_id_4.clone(),
						parent_id: None,
						group_data: data_4,
						decrypted_group_keys: HashMap::from([(user.user_id.to_string(), group_4_data_for_creator)]),
					},
				])
			}
		})
		.await;
}

#[tokio::test]
async fn test_10_access_group_3_as_parent()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let user = &users[0];
	let jwt = &user.user_data.jwt;

	let parent_group = &groups[1];
	let group = &groups[2];
	let keys = &parent_group
		.decrypted_group_keys
		.get(user.user_id.as_str())
		.unwrap()[0];
	let private_key = &keys.private_group_key;

	let (data_3, group_3_data_for_creator) = get_group(secret_token.as_str(), jwt, &group.group_id, private_key, false).await;

	assert_eq!(data_3.group_id, group.group_id);
	assert_eq!(group_3_data_for_creator.len(), 1);
}

#[tokio::test]
async fn test_11_access_group_3_as_parent_again_to_check_the_cache()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let user = &users[0];
	let jwt = &user.user_data.jwt;

	let parent_group = &groups[1];
	let group = &groups[2];
	let keys = &parent_group
		.decrypted_group_keys
		.get(user.user_id.as_str())
		.unwrap()[0];
	let private_key = &keys.private_group_key;

	let (data_3, group_3_data_for_creator) = get_group(secret_token.as_str(), jwt, &group.group_id, private_key, false).await;

	assert_eq!(data_3.group_id, group.group_id);
	assert_eq!(group_3_data_for_creator.len(), 1);
}

#[tokio::test]
async fn test_12_connect_group_to_group_one()
{
	//connect 4. group to group 1.
	//use the service because connecting to a non connected group is not allowed from the controller

	let app_data = APP_TEST_STATE.get().unwrap().read().await;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let user = &users[0];

	let group = &groups[0];

	let group_to_connect_to = &groups[3];

	server_api::start().await;
	//change path for sqlite because test files are executed from a different dir than the normal api
	std::env::set_var("DB_PATH", std::env::var("DB_PATH_TEST").unwrap());

	//prepare the keys

	let user_keys = group
		.decrypted_group_keys
		.get(user.user_id.as_str())
		.unwrap();

	let mut group_keys_ref = vec![];

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	//get the public key of the con group 2 from the service
	let key = server_api::sentc_group_service::get_public_key_data(app_data.app_id.to_string(), &group_to_connect_to.group_id)
		.await
		.unwrap();

	let server_out = ServerOutput {
		status: true,
		err_msg: None,
		err_code: None,
		result: Some(key),
	};
	let server_out = server_out.to_string().unwrap();

	let group_to_invite_public_key = sentc_crypto::util::public::import_public_key_from_string_into_format(&server_out).unwrap();

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(&group_to_invite_public_key, &group_keys_ref, false).unwrap();

	let input = GroupKeysForNewMemberServerInput::from_string(&invite).unwrap();

	let group_data = server_api::sentc_group_entities::InternalGroupDataComplete {
		group_data: server_api::sentc_group_entities::InternalGroupData {
			id: group.group_id.to_string(),
			app_id: app_data.app_id.to_string(),
			parent: None,
			time: 0,
			invite: 1,                //must be true
			is_connected_group: true, //no err when calling this fn directly
		},
		user_data: server_api::sentc_group_entities::InternalUserGroupData {
			user_id: "".to_string(),
			real_user_id: "".to_string(),
			joined_time: 0,
			rank: 0, //can be admin rank
			get_values_from_parent: None,
			get_values_from_group_as_member: None,
		},
	};

	server_api::sentc_group_user_service::invite_auto(
		&group_data,
		input,
		&group_to_connect_to.group_id,
		server_api::sentc_group_user_service::NewUserType::Group,
	)
	.await
	.unwrap();
}

#[tokio::test]
async fn test_13_access_group_3_as_parent_from_connected_group()
{
	//access the 3. group from the connected group via parent

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let user = &users[0];
	let jwt = &user.user_data.jwt;

	let parent_group = &groups[1];
	let group = &groups[2];
	let keys = &parent_group
		.decrypted_group_keys
		.get(user.user_id.as_str())
		.unwrap()[0];
	let private_key = &keys.private_group_key;

	let group_to_access = &groups[3];

	let (data_3, group_3_data_for_creator) = get_group_from_group_as_member(
		secret_token.as_str(),
		jwt,
		&group.group_id,
		&group_to_access.group_id,
		private_key,
	)
	.await;

	assert_eq!(data_3.group_id, group.group_id);
	assert_eq!(group_3_data_for_creator.len(), 1);
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

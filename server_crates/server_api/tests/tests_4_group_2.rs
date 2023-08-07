use std::collections::HashMap;

use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use rustgram_server_util::error::ServerErrorCodes;
use sentc_crypto::entities::group::GroupKeyData;
use sentc_crypto::entities::user::UserDataInt;
use sentc_crypto::sdk_utils::error::SdkUtilError;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::SdkError;
use sentc_crypto_common::group::{
	CreateData,
	GroupAcceptJoinReqServerOutput,
	GroupCreateOutput,
	GroupInviteReqList,
	GroupInviteServerOutput,
	GroupJoinReqList,
	GroupKeysForNewMemberServerInput,
	GroupServerData,
	ListGroups,
};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::{GroupId, ServerOutput, UserId};
use server_api::util::api_res::ApiErrorCodes;
use server_dashboard_common::app::AppRegisterOutput;
use server_dashboard_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
	add_group_by_invite,
	add_user_by_invite,
	add_user_by_invite_as_group_as_member,
	auth_header,
	create_app,
	create_child_group_from_group_as_member,
	create_group,
	create_test_customer,
	create_test_user,
	customer_delete,
	delete_app,
	delete_user,
	done_key_rotation,
	get_group,
	get_group_from_group_as_member,
	get_server_error_from_normal_res,
	get_url,
	key_rotation,
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
static CONNECTED_GROUP_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();
static CHILD_GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();
static CONNECTED_CHILDREN_GROUP_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();

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
3. a parent group and one child group which got access to a connected group

create connected groups:
1. the group to connect (created from group 0)
2. a group to connect (but from user and connect it later), with a child (connect group 1 and 2)

At the end connect a child from one group to a parent for another group and
check if user from one parent got access at the other parent
 */
#[tokio::test]
async fn test_01_create_groups()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;

	let n = 4;
	let mut groups = Vec::with_capacity(n);

	for i in 0..n {
		let user = &users[i];
		let public_key = &user.user_data.user_keys[0].public_key;
		let private_key = &user.user_data.user_keys[0].private_key;
		let jwt = &user.user_data.jwt;

		let group_id = create_group(secret_token.as_str(), public_key, None, jwt).await;

		let (data, group_data_for_creator) = get_group(secret_token.as_str(), jwt, group_id.as_str(), private_key, false).await;

		assert!(!data.is_connected_group);

		let mut decrypted_group_keys = HashMap::new();

		decrypted_group_keys.insert(user.user_id.to_string(), group_data_for_creator);

		groups.push(GroupState {
			group_id,
			group_member: vec![user.user_id.to_string()],
			decrypted_group_keys,
		});
	}

	//create now the children:
	// one for group 2 to test access from parent to connected group 1
	//one for group 2 and connect later group 1 and 2 to test access via connected parent group

	let mut children = Vec::with_capacity(2);

	let groups_and_users = vec![(1, 1), (2, 2)];

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
			group_member: vec![user.user_id.to_string()],
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

#[tokio::test]
async fn test_10_create_a_connected_group_from_a_group()
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
	let group_1_public_key = &group_1
		.decrypted_group_keys
		.get(&group_1.group_member[0])
		.unwrap()[0]
		.public_group_key;

	let url = get_url("api/v1/group".to_owned() + "/" + group_1.group_id.as_str() + "/connected");

	let group_input = sentc_crypto::group::prepare_create(group_1_public_key).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(&creator_group_1.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: GroupCreateOutput = handle_server_response(&body).unwrap();

	//now get the group
	let data = get_group_from_group_as_member(
		secret_token,
		&creator_group_1.user_data.jwt,
		&out.group_id,
		&group_1.group_id,
		group_1_private_key,
	)
	.await;

	assert!(data.0.is_connected_group);

	let mut decrypted_group_keys = HashMap::new();

	decrypted_group_keys.insert(creator_group_1.user_id.to_string(), data.1);

	CONNECTED_GROUP_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(vec![GroupState {
					group_id: out.group_id.clone(),
					group_member: vec![creator_group_1.user_id.to_string()],
					decrypted_group_keys,
				}])
			}
		})
		.await;
}

#[tokio::test]
async fn test_11_not_connect_normal_group_to_normal_group_by_invite()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	//try to connected group 1 and group 2 (which are both normal groups).
	//should fail

	let creator_group_1 = &users[0];
	let group_1 = &groups[0];
	let user_keys = group_1
		.decrypted_group_keys
		.get(&group_1.group_member[0])
		.unwrap();

	let group_2 = &groups[0];

	//get the public key of the group 2
	let url = get_url("api/v1/group/".to_owned() + &group_2.group_id + "/public_key");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator_group_1.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//prepare invite for group 2
	let mut group_keys_ref = vec![];

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let group_to_invite_public_key = sentc_crypto::util::public::import_public_key_from_string_into_format(&body).unwrap();

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(&group_to_invite_public_key, &group_keys_ref, false, None).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &group_1.group_id + "/invite_group_auto/" + &group_2.group_id);

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(&creator_group_1.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.body(invite)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	match handle_server_response::<GroupInviteServerOutput>(&body) {
		Ok(_) => {
			panic!("Should be an error");
		},
		Err(e) => {
			match e {
				SdkError::Util(SdkUtilError::ServerErr(s, _)) => {
					assert_eq!(s, ApiErrorCodes::GroupJoinAsConnectedGroup.get_int_code());
				},
				_ => panic!("Should be server error"),
			}
		},
	}
}

#[tokio::test]
async fn test_12_not_connect_normal_group_to_normal_group_by_join()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	//try to connected group 1 and group 2 (which are both normal groups).
	//should fail

	let group = &groups[1];

	let group_to_invite = &groups[0];
	let user_to_invite = &users[0];

	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/join_req/" + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&user_to_invite.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, ApiErrorCodes::GroupJoinAsConnectedGroup.get_int_code());
}

#[tokio::test]
async fn test_13_not_invite_connected_group_as_member()
{
	//try to invite con group 1 to group 2 (because group 1 is already connected and member)
	//should fail

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &groups[1];
	let creator = &users[1];
	let user_keys = group
		.decrypted_group_keys
		.get(&group.group_member[0])
		.unwrap();

	let con_group = &con_groups[0];

	//now try to invite
	//get the public key of the con group
	let url = get_url("api/v1/group/".to_owned() + &con_group.group_id + "/public_key");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let mut group_keys_ref = vec![];

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let group_to_invite_public_key = sentc_crypto::util::public::import_public_key_from_string_into_format(&body).unwrap();

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(&group_to_invite_public_key, &group_keys_ref, false, None).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &group.group_id + "/invite_group_auto/" + &con_group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.body(invite)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	match handle_server_response::<GroupInviteServerOutput>(&body) {
		Ok(_) => {
			panic!("Should be an error");
		},
		Err(e) => {
			match e {
				SdkError::Util(SdkUtilError::ServerErr(s, _)) => {
					assert_eq!(s, ApiErrorCodes::GroupJoinAsConnectedGroup.get_int_code());
				},
				_ => panic!("Should be server error"),
			}
		},
	}
}

#[tokio::test]
async fn test_14_not_send_join_req_as_connected_group()
{
	//try to join group 2 from con group 1
	//should fail

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &groups[1];

	let con_group = &con_groups[0];
	let con_group_user = &users[0];
	let con_group_access = &groups[0];

	let url = get_url("api/v1/group/".to_owned() + &con_group.group_id + "/join_req/" + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&con_group_user.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &con_group_access.group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, ApiErrorCodes::GroupJoinAsConnectedGroup.get_int_code());
}

#[tokio::test]
async fn test_14_z_connect_conn_group_to_other_conn_group_by_service()
{
	let app_data = APP_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let conn_group1 = &con_groups[0];

	server_api::start().await;
	//change path for sqlite because test files are executed from a different dir than the normal api
	std::env::set_var("DB_PATH", std::env::var("DB_PATH_TEST").unwrap());

	//create a 2nd connected group to test
	let group_1 = &groups[0];
	let group_1_public_key = &group_1
		.decrypted_group_keys
		.get(&group_1.group_member[0])
		.unwrap()[0]
		.public_group_key;

	let group_input = sentc_crypto::group::prepare_create(group_1_public_key).unwrap();

	let (conn_group_2_id, _) = server_api::sentc_group_service::create_group(
		app_data.app_id.to_string(),
		"",
		CreateData::from_string(&group_input).unwrap(),
		server_api_common::group::GROUP_TYPE_NORMAL,
		None,
		Some(0),
		Some(group_1.group_id.clone()),
		true,
	)
	.await
	.unwrap();

	//prepare the keys

	let user_keys = conn_group1
		.decrypted_group_keys
		.get(&conn_group1.group_member[0])
		.unwrap();

	let mut group_keys_ref = vec![];

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	//get the public key of the con group 2 from the service
	let key = server_api::sentc_group_service::get_public_key_data(app_data.app_id.to_string(), &conn_group_2_id)
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

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(&group_to_invite_public_key, &group_keys_ref, false, None).unwrap();

	let input = GroupKeysForNewMemberServerInput::from_string(&invite).unwrap();

	let group_data = server_api_common::group::group_entities::InternalGroupDataComplete {
		group_data: server_api_common::group::group_entities::InternalGroupData {
			id: conn_group1.group_id.to_string(),
			app_id: app_data.app_id.to_string(),
			parent: None,
			time: 0,
			invite: 1,                //must be true
			is_connected_group: true, //no err when calling this fn directly
		},
		user_data: server_api_common::group::group_entities::InternalUserGroupData {
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
		&conn_group_2_id,
		server_api::sentc_group_user_service::NewUserType::Group,
	)
	.await
	.unwrap();
}

#[tokio::test]
async fn test_15_connected_another_group_to_connected_group_by_invite()
{
	//connect the 2nd group with the connected group from group 0

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group_from_con = &groups[0];
	let group_to_connect = &con_groups[0];
	let creator_group_to_connect = &users[0];

	let user_keys = group_to_connect
		.decrypted_group_keys
		.get(&group_to_connect.group_member[0])
		.unwrap();

	let group_1 = &groups[1];
	let creator_group_1 = &users[1];
	let group_1_private_key = &group_1
		.decrypted_group_keys
		.get(&group_1.group_member[0])
		.unwrap()[0]
		.private_group_key;
	let group_1_exported_public_key = &group_1
		.decrypted_group_keys
		.get(&group_1.group_member[0])
		.unwrap()[0]
		.exported_public_key;

	let data_1 = add_group_by_invite(
		secret_token,
		&creator_group_to_connect.user_data.jwt,
		&group_to_connect.group_id,
		user_keys,
		&group_1.group_id,
		group_1_exported_public_key,
		&creator_group_1.user_data.jwt,
		group_1_private_key,
		Some(&group_from_con.group_id),
	)
	.await;

	assert_eq!(data_1.0.rank, 4);
}

#[tokio::test]
async fn test_16_connected_child_group()
{
	//connected child group of group 3 to con group 1
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let children = CHILD_GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group_from_con = &groups[0];
	let group_to_connect = &con_groups[0];
	let creator_group_to_connect = &users[0];

	let user_keys = group_to_connect
		.decrypted_group_keys
		.get(&group_to_connect.group_member[0])
		.unwrap();

	let child_2 = &children[1];
	let creator_group_1 = &users[2];
	let group_1_private_key = &child_2
		.decrypted_group_keys
		.get(&child_2.group_member[0])
		.unwrap()[0]
		.private_group_key;
	let group_1_exported_public_key = &child_2
		.decrypted_group_keys
		.get(&child_2.group_member[0])
		.unwrap()[0]
		.exported_public_key;

	let data_1 = add_group_by_invite(
		secret_token,
		&creator_group_to_connect.user_data.jwt,
		&group_to_connect.group_id,
		user_keys,
		&child_2.group_id,
		group_1_exported_public_key,
		&creator_group_1.user_data.jwt,
		group_1_private_key,
		Some(&group_from_con.group_id),
	)
	.await;

	assert_eq!(data_1.0.rank, 4);
}

#[tokio::test]
async fn test_17_access_con_group_from_a_parent()
{
	//access the con group 1 from parent of child group 1
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let children = CHILD_GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group_to_connect = &con_groups[0];

	let child_2 = &children[1];
	let creator_group_1 = &users[2];
	let child_2_private_key = &child_2
		.decrypted_group_keys
		.get(&child_2.group_member[0])
		.unwrap()[0]
		.private_group_key;

	let data = get_group_from_group_as_member(
		secret_token,
		&creator_group_1.user_data.jwt,
		&group_to_connect.group_id,
		&child_2.group_id, //access this group from the child, but user is member of the parent -> should work
		child_2_private_key,
	)
	.await;

	assert_eq!(data.0.rank, 4);
}

#[tokio::test]
async fn test_18_create_child_group_from_con_group()
{
	//to test access of the children from a con group
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group_from_con = &groups[0];
	let group_to_connect = &con_groups[0];
	let creator_group_to_connect = &users[0];

	let con_group = &con_groups[0];

	let keys = &con_group
		.decrypted_group_keys
		.get(&con_group.group_member[0])
		.unwrap()[0];

	let public_key = &keys.public_group_key;
	let private_key = &keys.private_group_key;
	let jwt = &creator_group_to_connect.user_data.jwt;

	//this user should got the right rank
	let group_id = create_child_group_from_group_as_member(
		secret_token.as_str(),
		public_key,
		&group_to_connect.group_id,
		jwt,
		&group_from_con.group_id,
	)
	.await;

	let data = get_group_from_group_as_member(
		secret_token,
		&creator_group_to_connect.user_data.jwt,
		&group_id,
		&group_from_con.group_id,
		private_key,
	)
	.await;

	assert!(data.0.is_connected_group);

	let mut decrypted_group_keys = HashMap::new();

	decrypted_group_keys.insert(creator_group_to_connect.user_id.to_string(), data.1);

	CONNECTED_CHILDREN_GROUP_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(vec![GroupState {
					group_id,
					group_member: vec![creator_group_to_connect.user_id.to_string()],
					decrypted_group_keys,
				}])
			}
		})
		.await;
}

#[tokio::test]
async fn test_19_rank_from_member_of_connected_group()
{
	//auto invite a member to the main group and then access the connected group
	//check the rank of this member. this should not be the creator rank but the rank of the connected group

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let creator_group_1 = &users[0];
	let group = &groups[0];
	let group_to_connect = &con_groups[0];
	let user_to_invite = &users[6];

	let user_keys = group
		.decrypted_group_keys
		.get(creator_group_1.user_id.as_str())
		.unwrap();

	add_user_by_invite(
		secret_token,
		&creator_group_1.user_data.jwt,
		&group.group_id,
		user_keys,
		&user_to_invite.user_id,
		&user_to_invite.user_data.jwt,
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&user_to_invite.user_data.user_keys[0].private_key,
	)
	.await;

	//now check the access and the rank in the connected group
	let group_1_private_key = &group
		.decrypted_group_keys
		.get(&group.group_member[0])
		.unwrap()[0]
		.private_group_key;

	let data = get_group_from_group_as_member(
		secret_token,
		&user_to_invite.user_data.jwt,
		&group_to_connect.group_id,
		&group.group_id,
		group_1_private_key,
	)
	.await;

	//should be 4 not 0 (0 because the connected group is also the creator),
	// but 4 because user is also rank 4 in the connected group
	assert_eq!(data.0.rank, 4);
}

//__________________________________________________________________________________________________

#[tokio::test]
async fn test_20_key_rotation()
{
	//do a key rotation in a connected group and check if group 1 and 2 got new keys
	//this test is about to test the key rotation of group as member

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let mut con_groups = CONNECTED_GROUP_STATE.get().unwrap().write().await;

	let group_1 = &groups[0];
	let group_1_public_key = &group_1
		.decrypted_group_keys
		.get(&group_1.group_member[0])
		.unwrap()[0]
		.public_group_key;
	let group_1_private_key = &group_1
		.decrypted_group_keys
		.get(&group_1.group_member[0])
		.unwrap()[0]
		.private_group_key;

	let group_2 = &groups[1];
	let creator_group_2 = &users[1];
	let group_2_private_key = &group_2
		.decrypted_group_keys
		.get(&group_2.group_member[0])
		.unwrap()[0]
		.private_group_key;

	let group_to_connect = &con_groups[0];
	let creator_group_to_connect = &users[0];
	let pre_group_key = &group_to_connect
		.decrypted_group_keys
		.get(&creator_group_to_connect.user_id)
		.unwrap()[0]
		.group_key;

	//add user 5 as member of the connected group to check key rotation as a direct member
	let direct_user = &users[4];

	let user_keys = group_to_connect
		.decrypted_group_keys
		.get(creator_group_to_connect.user_id.as_str())
		.unwrap();

	add_user_by_invite_as_group_as_member(
		secret_token,
		&creator_group_to_connect.user_data.jwt,
		&group_to_connect.group_id,
		&group_1.group_id,
		user_keys,
		&direct_user.user_id,
		&direct_user.user_data.jwt,
		&direct_user.user_data.user_keys[0].exported_public_key,
		&direct_user.user_data.user_keys[0].private_key,
	)
	.await;

	//do the rotation for the connected group
	//the invoker is group 1
	key_rotation(
		secret_token,
		&creator_group_to_connect.user_data.jwt,
		&group_to_connect.group_id,
		pre_group_key,
		group_1_public_key,
		group_1_private_key,
		Some(&group_1.group_id),
	)
	.await;

	//key rotation as direct member
	let mut direct_member_data = get_group(
		secret_token,
		&direct_user.user_data.jwt,
		&group_to_connect.group_id,
		&direct_user.user_data.user_keys[0].private_key,
		true,
	)
	.await;

	assert!(direct_member_data.0.key_update);

	let mut direct_member_new_keys = done_key_rotation(
		secret_token,
		&direct_user.user_data.jwt,
		&group_to_connect.group_id,
		pre_group_key,
		&direct_user.user_data.user_keys[0].public_key,
		&direct_user.user_data.user_keys[0].private_key,
		None,
	)
	.await;

	//key rotation as a connected group
	let data_2 = get_group_from_group_as_member(
		secret_token,
		&creator_group_2.user_data.jwt,
		&group_to_connect.group_id,
		&group_2.group_id,
		group_2_private_key,
	)
	.await;

	assert!(data_2.0.key_update);

	done_key_rotation(
		secret_token,
		&creator_group_2.user_data.jwt,
		&group_to_connect.group_id,
		pre_group_key,
		&group_2
			.decrypted_group_keys
			.get(&creator_group_2.user_id)
			.unwrap()[0]
			.public_group_key,
		&group_2
			.decrypted_group_keys
			.get(&creator_group_2.user_id)
			.unwrap()[0]
			.private_group_key,
		Some(&group_2.group_id),
	)
	.await;

	//add the new keys for the new user to the con group state
	direct_member_new_keys.append(&mut direct_member_data.1);

	con_groups[0]
		.decrypted_group_keys
		.insert(direct_user.user_id.to_string(), direct_member_new_keys);
}

#[tokio::test]
async fn test_21_key_rotation_with_multiple_keys()
{
	/*
	do a key rotation in group 1. then do a rotation in connected group

	later check if there is only one new key to rotate and not 2
	(which would be the case if not only the newest key was used but all group keys)
	 */

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let mut groups = GROUP_TEST_STATE.get().unwrap().write().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group_1 = &groups[0];
	let creator_group_1 = &users[0];
	let pre_group_key_group_1 = &group_1
		.decrypted_group_keys
		.get(&group_1.group_member[0])
		.unwrap()[0]
		.group_key;

	let con_group_member = &users[4];
	let group_to_connect = &con_groups[0];
	let pre_group_key_group_connected = &group_to_connect
		.decrypted_group_keys
		.get(&con_group_member.user_id)
		.unwrap()[0]
		.group_key;

	//key rotation of group 1
	let (_, keys) = key_rotation(
		secret_token,
		&creator_group_1.user_data.jwt,
		&group_1.group_id,
		pre_group_key_group_1,
		&creator_group_1.user_data.user_keys[0].public_key,
		&creator_group_1.user_data.user_keys[0].private_key,
		None,
	)
	.await;

	assert_eq!(keys.len(), 2);

	//just get the key rotation data (not done it) to check how many keys are in (should be just one)
	key_rotation(
		secret_token,
		&con_group_member.user_data.jwt,
		&group_to_connect.group_id,
		pre_group_key_group_connected,
		&con_group_member.user_data.user_keys[0].public_key,
		&con_group_member.user_data.user_keys[0].private_key,
		None,
	)
	.await;

	let url = get_url("api/v1/group/".to_owned() + group_to_connect.group_id.as_str() + "/key_rotation");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator_group_1.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &group_1.group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<sentc_crypto::sdk_common::group::KeyRotationInput> = handle_server_response(body.as_str()).unwrap();

	assert_eq!(out.len(), 1);

	groups[0]
		.decrypted_group_keys
		.insert(creator_group_1.user_id.to_string(), keys);
}

#[tokio::test]
async fn test_22_kick_group_as_member()
{
	//kick group 2 from connected group (which we connected to the con group at test 15)

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &con_groups[0];
	let creator = &users[0];
	let group_to_access = &groups[0];

	let group_to_kick = &groups[1];
	let user_to_kick = &users[1];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/kick/" + &group_to_kick.group_id);

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &group_to_access.group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();

	//user should not get group data
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user_to_kick.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &group_to_kick.group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<ServerSuccessOutput>::from_string(body.as_str()).unwrap();

	assert!(!out.status);
}

#[tokio::test]
async fn test_23_invite_another_group()
{
	//invite send from the group to connect to group 2
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &con_groups[0];
	let creator = &users[0];
	let group_to_access = &groups[0];

	let group_to_invite = &groups[1];
	let user_to_invite = &users[1];

	let keys = group
		.decrypted_group_keys
		.get(&group.group_member[0])
		.unwrap();

	let mut group_keys_ref = vec![];

	for decrypted_group_key in keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	//fetch the public key data like user for a group which should connect to group
	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/public_key");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let group_to_invite_public_key = sentc_crypto::util::public::import_public_key_from_string_into_format(&body).unwrap();

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(&group_to_invite_public_key, &group_keys_ref, false, None).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &group.group_id + "/invite_group/" + &group_to_invite.group_id);

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &group_to_access.group_id)
		.body(invite)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let invite_res: GroupInviteServerOutput = handle_server_response(&body).unwrap();

	assert_eq!(invite_res.session_id, None);

	//get the invite list for the group

	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/invite/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&user_to_invite.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let invites: Vec<GroupInviteReqList> = handle_server_response(&body).unwrap();

	assert_eq!(invites.len(), 1);

	assert_eq!(&invites[0].group_id, &group.group_id);
}

#[tokio::test]
async fn test_24_reject_invite()
{
	//should be the same as normal invite
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &con_groups[0];

	let group_to_invite = &groups[1];
	let user_to_invite = &users[1];

	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/invite/" + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(&user_to_invite.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//group should not be on the invite list
	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/invite/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&user_to_invite.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let invites: Vec<GroupInviteReqList> = handle_server_response(&body).unwrap();

	assert_eq!(invites.len(), 0);
}

#[tokio::test]
async fn test_25_accept_invite()
{
	//invite the group again to accept the invite

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &con_groups[0];
	let creator = &users[0];
	let group_to_access = &groups[0];

	let group_to_invite = &groups[1];
	let user_to_invite = &users[1];
	let group_to_invite_private_key = &group_to_invite
		.decrypted_group_keys
		.get(&user_to_invite.user_id)
		.unwrap()[0]
		.private_group_key;

	let keys = group
		.decrypted_group_keys
		.get(&group.group_member[0])
		.unwrap();

	let mut group_keys_ref = vec![];

	for decrypted_group_key in keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	//fetch the public key data like user for a group which should connect to group
	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/public_key");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let group_to_invite_public_key = sentc_crypto::util::public::import_public_key_from_string_into_format(&body).unwrap();

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(&group_to_invite_public_key, &group_keys_ref, false, None).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &group.group_id + "/invite_group/" + &group_to_invite.group_id);

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &group_to_access.group_id)
		.body(invite)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let _invite_res: GroupInviteServerOutput = handle_server_response(&body).unwrap();

	//now accept the invite
	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/invite/" + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&user_to_invite.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//user of the invited group should access the group
	let _data = get_group_from_group_as_member(
		secret_token,
		&user_to_invite.user_data.jwt,
		&group.group_id,
		&group_to_invite.group_id,
		group_to_invite_private_key,
	)
	.await;
}

#[tokio::test]
async fn test_26_not_send_join_req_if_group_is_already_group_member()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &con_groups[0];

	let group_to_invite = &groups[1];
	let user_to_invite = &users[1];

	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/join_req/" + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&user_to_invite.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, ApiErrorCodes::GroupUserExists.get_int_code());
}

#[tokio::test]
async fn test_27_join_req_to_join_a_group()
{
	//use group 3 here to test join req, so we don't need to kick the other group again

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &con_groups[0];

	let group_to_invite = &groups[3];
	let user_to_invite = &users[3];

	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/join_req/" + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&user_to_invite.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_28_sent_join_req_for_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &con_groups[0];

	let group_to_invite = &groups[3];
	let user_to_invite = &users[3];

	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/joins/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user_to_invite.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<GroupInviteReqList> = handle_server_response(&body).unwrap();

	//should get the join req to this group
	assert_eq!(out.len(), 1);
	assert_eq!(out[0].group_id, group.group_id);
}

#[tokio::test]
async fn test_29_delete_sent_join_req()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &con_groups[0];

	let group_to_invite = &groups[3];
	let user_to_invite = &users[3];

	//delete the req
	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/joins/" + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(&user_to_invite.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//should not get it from the list
	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/joins/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user_to_invite.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<GroupInviteReqList> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 0);

	//send the req again for the other tests

	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/join_req/" + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&user_to_invite.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_30_reject_join_req_from_group()
{
	//reject the join req from group 3

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &con_groups[0];
	let creator = &users[0];
	let group_to_access = &groups[0];

	let group_to_invite = &groups[3];

	//first get the join reqs

	let url = get_url("api/v1/group/".to_owned() + &group.group_id + "/join_req/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &group_to_access.group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<GroupJoinReqList> = handle_server_response(&body).unwrap();

	assert_eq!(list.len(), 1);

	assert_eq!(list[0].user_id, group_to_invite.group_id);
	assert_eq!(list[0].user_type, 2); //group as member not user

	//now reject the join req, should work like normal user

	let url = get_url("api/v1/group/".to_owned() + &group.group_id + "/join_req/" + &list[0].user_id);

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &group_to_access.group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_31_accept_join_req_from_group()
{
	//send the join req again
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &con_groups[0];
	let creator = &users[0];
	let group_to_access = &groups[0];

	let group_to_invite = &groups[3];
	let user_to_invite = &users[3];
	let group_to_invite_private_key = &group_to_invite
		.decrypted_group_keys
		.get(&user_to_invite.user_id)
		.unwrap()[0]
		.private_group_key;

	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/join_req/" + &group.group_id);

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&user_to_invite.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//accept the join req
	let keys = group
		.decrypted_group_keys
		.get(&group.group_member[0])
		.unwrap();

	let mut group_keys_ref = vec![];

	for decrypted_group_key in keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	//fetch the public key data like user for a group which should connect to group
	let url = get_url("api/v1/group/".to_owned() + &group_to_invite.group_id + "/public_key");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let group_to_invite_public_key = sentc_crypto::util::public::import_public_key_from_string_into_format(&body).unwrap();

	let join = sentc_crypto::group::prepare_group_keys_for_new_member(&group_to_invite_public_key, &group_keys_ref, false, None).unwrap();

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req/" + &group_to_invite.group_id);

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &group_to_access.group_id)
		.body(join)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let join_res: GroupAcceptJoinReqServerOutput = handle_server_response(body.as_str()).unwrap();
	assert_eq!(join_res.session_id, None);

	//should get the group
	let data = get_group_from_group_as_member(
		secret_token,
		&user_to_invite.user_data.jwt,
		&group.group_id,
		&group_to_invite.group_id,
		group_to_invite_private_key,
	)
	.await;

	//should be the lowest rank for joined member
	assert_eq!(data.0.rank, 4);
}

#[tokio::test]
async fn test_32_not_leave_groups_without_rights()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let public_token_str = &APP_TEST_STATE.get().unwrap().read().await.public_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	//create a new user and set it as a member in the group to live
	let (user_id, key_data) = create_test_user(secret_token, public_token_str, "hi_123", "123").await;

	let group = &groups[3];
	let creator = &users[3];
	let group_to_leave = &con_groups[0];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	add_user_by_invite(
		secret_token,
		&creator.user_data.jwt,
		&group.group_id,
		user_keys,
		&user_id,
		&key_data.jwt,
		&key_data.user_keys[0].exported_public_key,
		&key_data.user_keys[0].private_key,
	)
	.await;

	//now try to leave the group as member
	let url = get_url("api/v1/group/".to_owned() + group_to_leave.group_id.as_str() + "/leave");
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(key_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", group.group_id.as_str())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//delete the test user before checking because after the user wont get deleted when there are still errors
	delete_user(secret_token, &user_id).await;

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, ApiErrorCodes::GroupUserRank.get_int_code());
}

#[tokio::test]
async fn test_33_get_all_groups_to_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &con_groups[0];

	let group_to_invite = &groups[3];
	let user_to_invite = &users[3];

	let url = get_url("api/v1/group/".to_owned() + group_to_invite.group_id.as_str() + "/all/0/none");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user_to_invite.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListGroups> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].group_id, group.group_id);
}

#[tokio::test]
async fn test_34_leave_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;
	let con_groups = CONNECTED_GROUP_STATE.get().unwrap().read().await;

	let group = &groups[3];
	let creator = &users[3];
	let group_to_leave = &con_groups[0];

	let url = get_url("api/v1/group/".to_owned() + group_to_leave.group_id.as_str() + "/leave");
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", group.group_id.as_str())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();

	//this user should not get the group data
	let url = get_url("api/v1/group/".to_owned() + group_to_leave.group_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", group.group_id.as_str())
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::BAD_REQUEST);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<GroupServerData>::from_string(body.as_str()).unwrap();

	assert!(!out.status);
	assert_eq!(out.err_code.unwrap(), ApiErrorCodes::GroupAccess.get_int_code());
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

	let customer_jwt = &CUSTOMER_TEST_STATE.get().unwrap().read().await.verify.jwt;

	delete_app(customer_jwt, app.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

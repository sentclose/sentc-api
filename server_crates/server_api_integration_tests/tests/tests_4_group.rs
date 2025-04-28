use std::collections::HashMap;
use std::time::Duration;

use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use sentc_crypto::sdk_common::group::{GroupAcceptJoinReqServerOutput, GroupInviteServerOutput};
use sentc_crypto::sdk_utils::error::SdkUtilError;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::SdkError;
use sentc_crypto_common::group::{
	GroupChangeRankServerInput,
	GroupChildrenList,
	GroupCreateOutput,
	GroupDataCheckUpdateServerOutput,
	GroupInviteReqList,
	GroupJoinReqList,
	GroupKeysForNewMemberServerInput,
	GroupLightServerData,
	GroupServerData,
	GroupUserListItem,
	KeyRotationStartServerOutput,
	ListGroups,
};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::{GroupId, ServerOutput, UserId};
use server_dashboard_common::app::AppRegisterOutput;
use server_dashboard_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
	add_user_by_invite,
	auth_header,
	create_app,
	create_group,
	create_test_customer,
	create_test_user,
	customer_delete,
	decrypt_group_hmac_keys,
	delete_app,
	delete_user,
	done_key_rotation,
	get_base_url,
	get_group,
	get_server_error_from_normal_res,
	get_url,
	init_user,
	key_rotation,
	user_key_rotation,
	TestGroup,
	TestGroupKeyData,
	TestUserDataInt,
};

mod test_fn;

pub struct UserState
{
	pub username: String,
	pub pw: String,
	pub user_id: UserId,
	pub user_data: TestUserDataInt,
}

pub struct GroupState
{
	pub group_id: GroupId,
	pub group_member: Vec<UserId>,
	pub decrypted_group_keys: HashMap<UserId, Vec<TestGroupKeyData>>,
}

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerDoneLoginOutput>> = OnceCell::const_new();
static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<Vec<UserState>>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<GroupState>> = OnceCell::const_new();
static CHILD_GROUP_TEST_STATE: OnceCell<RwLock<GroupState>> = OnceCell::const_new();

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

	GROUP_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(GroupState {
					group_id: "".to_string(),
					group_member: vec![],
					decrypted_group_keys: HashMap::new(),
				})
			}
		})
		.await;

	CHILD_GROUP_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(GroupState {
					group_id: "".to_string(),
					group_member: vec![],
					decrypted_group_keys: HashMap::new(),
				})
			}
		})
		.await;
}

//__________________________________________________________________________________________________
//tests start

#[tokio::test]
async fn test_10_create_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let mut group = GROUP_TEST_STATE.get().unwrap().write().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let group_input = TestGroup::prepare_create(&creator.user_data.user_keys[0].public_key, None, Default::default()).unwrap();

	let url = get_url("api/v1/group".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	let out = ServerOutput::<GroupCreateOutput>::from_string(body.as_str()).unwrap();

	assert!(out.status);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	group.group_id = out.group_id;
}

#[tokio::test]
async fn test_11_get_group_data()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let mut group = GROUP_TEST_STATE.get().unwrap().write().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<GroupServerData>::from_string(body.as_str()).unwrap();

	assert!(out.status);
	assert_eq!(out.err_code, None);

	//check if the result is there
	let _out = out.result.unwrap();

	let data = sentc_crypto::group::get_group_data(body.as_str()).unwrap();

	let mut data_keys_arr = Vec::with_capacity(data.keys.len());

	for key in data.keys {
		data_keys_arr.push(TestGroup::decrypt_group_keys(&creator.user_data.user_keys[0].private_key, key, None).unwrap());
	}

	let hmac_keys = decrypt_group_hmac_keys(&data_keys_arr[0].group_key, data.hmac_keys);

	assert_eq!(hmac_keys.len(), 1);

	//user is the creator
	assert_eq!(data.rank, 0);

	group
		.decrypted_group_keys
		.insert(creator.user_id.to_string(), data_keys_arr);
}

#[tokio::test]
async fn test_11_x_get_group_light_data()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().write().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/light");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out: GroupLightServerData = handle_server_response(&body).unwrap();

	assert_eq!(out.rank, 0);
}

#[tokio::test]
async fn test_11_z_get_group_update()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/update_check");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out: GroupDataCheckUpdateServerOutput = handle_server_response(body.as_str()).unwrap();

	assert_eq!(out.rank, 0);
	assert!(!out.key_update);
}

#[tokio::test]
async fn test_12_create_child_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;
	let mut child_group = CHILD_GROUP_TEST_STATE.get().unwrap().write().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	//use here the public group key for the child group!
	let group_public_key = &group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap()[0]
		.public_group_key;
	let group_private_key = &group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap()[0]
		.private_group_key;

	let child_id = create_group(
		secret_token,
		group_public_key,
		Some(group.group_id.to_string()),
		creator.user_data.jwt.as_str(),
	)
	.await;

	let (data, keys) = get_group(
		secret_token,
		creator.user_data.jwt.as_str(),
		child_id.as_str(),
		group_private_key,
		false,
	)
	.await;

	assert_eq!(data.rank, 0);
	assert_eq!(data.group_id, child_id);
	assert_eq!(data.parent_group_id.unwrap(), group.group_id.to_string());

	child_group
		.decrypted_group_keys
		.insert(creator.user_id.to_string(), keys);

	child_group.group_id = child_id;

	//don't delete the child group to test if the parent group delete deletes all. delete the child
}

#[tokio::test]
async fn test_12_x_get_all_children_for_parent()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;
	let child_group = CHILD_GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let url = get_url("api/v1/group/".to_owned() + &group.group_id + "/children/0/none");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<GroupChildrenList> = handle_server_response(&body).unwrap();

	assert_eq!(list.len(), 1);
	assert_eq!(list[0].group_id, child_group.group_id);

	//______________________________________________________________________________________________
	//test page two
	let url = get_url("api/v1/group/".to_owned() + &group.group_id + "/children/" + list[0].time.to_string().as_str() + "/" + &list[0].group_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let list: Vec<GroupChildrenList> = handle_server_response(&body).unwrap();

	assert_eq!(list.len(), 0);
}

//__________________________________________________________________________________________________
//invite

#[tokio::test]
async fn test_13_stop_group_invite()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[0];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/change_invite");
	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();
}

#[tokio::test]
async fn test_14_not_send_invite_or_join_when_invite_is_disabled()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[1];

	//no join req
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req");
	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, 317);

	//no invite req
	let creator = &users[0];

	let user_to_invite = &users[1];

	let mut group_keys_ref = vec![];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = TestGroup::prepare_group_keys_for_new_member(
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&group_keys_ref,
		false,
		None,
	)
	.unwrap();

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/invite/" + user_to_invite.user_id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(invite)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	match handle_server_response::<GroupInviteServerOutput>(body.as_str()) {
		Ok(_) => panic!("Should be an error"),
		Err(e) => {
			match e {
				SdkError::Util(SdkUtilError::ServerErr(s, _)) => {
					assert_eq!(s, 317);
				},
				_ => panic!("should be server error"),
			}
		},
	}
}

#[tokio::test]
async fn test_15_enable_group_invite()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[0];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/change_invite");
	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();
}

#[tokio::test]
async fn test_16_invite_user()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];

	let user_to_invite = &users[1];

	let mut group_keys_ref = vec![];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = TestGroup::prepare_group_keys_for_new_member(
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&group_keys_ref,
		false,
		None,
	)
	.unwrap();

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/invite/" + user_to_invite.user_id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(invite)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let invite_res: GroupInviteServerOutput = handle_server_response(body.as_str()).unwrap();

	assert_eq!(invite_res.session_id, None);
}

#[tokio::test]
async fn test_17_not_invite_user_without_keys()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];

	let user_to_invite = &users[1];

	let mut group_keys_ref = vec![];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	//no keys -> must be an error
	let input = GroupKeysForNewMemberServerInput {
		keys: Vec::new(),
		rank: None,
		key_session: false,
	};

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/invite/" + user_to_invite.user_id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out = ServerOutput::<ServerSuccessOutput>::from_string(body.as_str()).unwrap();

	assert!(!out.status);
	assert_eq!(out.err_code, Some(303));
}

#[tokio::test]
async fn test_18_get_invite_for_user()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;

	let user_to_invite = &users[1];

	let url = get_url("api/v1/group/".to_owned() + "invite/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user_to_invite.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out = ServerOutput::<Vec<GroupInviteReqList>>::from_string(body.as_str()).unwrap();

	assert!(out.status);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].group_id.to_string(), group.group_id.to_string());
}

#[tokio::test]
async fn test_19_user_init_with_invites()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user_to_invite = &users[1];

	let out = init_user(
		secret_token,
		user_to_invite.user_data.jwt.as_str(),
		user_to_invite.user_data.refresh_token.as_str(),
	)
	.await;

	assert_eq!(out.invites.len(), 1);
}

#[tokio::test]
async fn test_20_accept_invite()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let mut group = GROUP_TEST_STATE.get().unwrap().write().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;

	let user_to_invite = &users[1];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/invite");

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(user_to_invite.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();

	//test get group as new user
	let data = get_group(
		secret_token,
		user_to_invite.user_data.jwt.as_str(),
		group.group_id.as_str(),
		&user_to_invite.user_data.user_keys[0].private_key,
		false,
	)
	.await;

	//should be normal user rank
	assert_eq!(data.0.rank, 4);

	group
		.decrypted_group_keys
		.insert(user_to_invite.user_id.to_string(), data.1);

	group.group_member.push(user_to_invite.user_id.to_string());
}

#[tokio::test]
async fn test_21_invite_user_an_reject_invite()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];

	let user_to_invite = &users[2];

	let mut group_keys_ref = vec![];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = TestGroup::prepare_group_keys_for_new_member(
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&group_keys_ref,
		false,
		None,
	)
	.unwrap();

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/invite/" + user_to_invite.user_id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(invite)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let invite_res: GroupInviteServerOutput = handle_server_response(body.as_str()).unwrap();
	assert_eq!(invite_res.session_id, None);

	//______________________________________________________________________________________________
	//no reject the invite

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/invite");

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(user_to_invite.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();

	//______________________________________________________________________________________________
	//the rejected user should not get the group data

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user_to_invite.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::BAD_REQUEST);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<GroupServerData>::from_string(body.as_str()).unwrap();

	assert!(!out.status);
	assert_eq!(out.err_code.unwrap(), 310);
}

//__________________________________________________________________________________________________
//leave group

#[tokio::test]
async fn test_22_not_leave_group_when_user_is_the_only_admin()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/leave");
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::BAD_REQUEST);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<ServerSuccessOutput>::from_string(body.as_str()).unwrap();
	assert!(!out.status);

	//should get the data without error
	let _data = get_group(
		secret_token,
		creator.user_data.jwt.as_str(),
		group.group_id.as_str(),
		&creator.user_data.user_keys[0].private_key,
		false,
	)
	.await;
}

#[tokio::test]
async fn test_23_leave_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[1];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/leave");
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();

	//this user should not get the group data
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::BAD_REQUEST);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<GroupServerData>::from_string(body.as_str()).unwrap();

	assert!(!out.status);
	assert_eq!(out.err_code.unwrap(), 310);
}

//__________________________________________________________________________________________________
//join req

#[tokio::test]
async fn test_24_join_req()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[1];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req");
	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();
}

#[tokio::test]
async fn test_25_get_join_req()
{
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	//get the first page
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req/" + "0/abc");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out: Vec<GroupJoinReqList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].user_id.to_string(), users[1].user_id.to_string());

	//should get the 2nd page without any join req
	let url = get_url(
		"api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req/" + out[0].time.to_string().as_str() + "/" + out[0].user_id.as_str(),
	);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out: Vec<GroupJoinReqList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_25_sent_join_req_for_user()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[1];

	let url = get_url("api/v1/group/joins/0/none".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<GroupInviteReqList> = handle_server_response(&body).unwrap();

	//should get the join req to this group
	assert_eq!(out.len(), 1);
	assert_eq!(out[0].group_id, group.group_id);

	//no join req for page 2
	let url = get_url("api/v1/group/joins/".to_owned() + out[0].time.to_string().as_str() + "/" + out[0].group_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<GroupInviteReqList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_26_send_join_req_aging()
{
	//this should not err because of insert ignored

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[1];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req");
	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();

	//should still get one join req
	let url = get_url("api/v1/group/joins/0/none".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<GroupInviteReqList> = handle_server_response(&body).unwrap();

	//should get the join req to this group
	assert_eq!(out.len(), 1);
	assert_eq!(out[0].group_id, group.group_id);

	//______________________________________________________________________________________________
	let creator = &users[0];

	//should still be this one join req
	//gets the first page
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req/" + "0/none");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out: Vec<GroupJoinReqList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].user_id.to_string(), users[1].user_id.to_string());
}

#[tokio::test]
async fn test_27_delete_join_req()
{
	//send the join req again for the next tests
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[1];

	let url = get_url("api/v1/group/joins/".to_owned() + &group.group_id);
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//no req on the list
	let url = get_url("api/v1/group/joins/0/none".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<GroupInviteReqList> = handle_server_response(&body).unwrap();

	//should get the join req to this group
	assert_eq!(out.len(), 0);

	//send again for the other tests
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req");
	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();
}

#[tokio::test]
async fn test_27_reject_join_req()
{
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	//get the first page
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req/" + users[1].user_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();
}

#[tokio::test]
async fn test_28_get_not_join_req_after_reject()
{
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	//get the first page
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req/" + "0/none");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<Vec<GroupJoinReqList>>::from_string(body.as_str()).unwrap();

	assert!(out.status);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	assert_eq!(out.len(), 0);

	//should not get the rejected join req in the sent join req list
	let user = &users[1];

	let url = get_url("api/v1/group/joins/0/none".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<GroupInviteReqList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_29_join_req_when_user_is_in_parent_group()
{
	//it should not make a join req from the creator of the parent group because they are in the parent as member
	let parent = GROUP_TEST_STATE.get().unwrap().read().await;
	let group = CHILD_GROUP_TEST_STATE.get().unwrap().read().await;
	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let creator = &users[0];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	let mut group_keys_ref = vec![];

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	//test auto invite
	let user_group_data_2 = add_user_by_invite(
		secret_token,
		&creator.user_data.jwt,
		&group.group_id,
		user_keys,
		&creator.user_id,
		&creator.user_data.jwt,
		&creator.user_data.user_keys[0].exported_public_key,
		&creator.user_data.user_keys[0].private_key,
	)
	.await;

	//access from the group directly
	assert_eq!(user_group_data_2.0.access_by_parent_group, None);
	assert_eq!(user_group_data_2.0.rank, 4); //rank 4 even got rank 0 in parent

	//remove user from the group
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/leave");
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();

	//test if still got access from the parent group
	let data = sentc_crypto::util_req_full::group::get_group(
		get_base_url(),
		secret_token,
		&creator.user_data.jwt,
		group.group_id.as_str(),
		None,
	)
	.await
	.unwrap();

	assert_eq!(data.access_by_parent_group, Some(parent.group_id.clone()));
	assert_eq!(data.rank, 0); //back to parent group rank
}

#[tokio::test]
async fn test_30_accept_join_req()
{
	//1. send the join req again, because we were rejecting the last one
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[1];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req");
	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();

	//______________________________________________________________________________________________
	//2. accept this join req
	let creator = &users[0];

	let user_to_accept = &users[1];

	let mut group_keys_ref = vec![];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let join = TestGroup::prepare_group_keys_for_new_member(
		&user_to_accept.user_data.user_keys[0].exported_public_key,
		&group_keys_ref,
		false,
		None,
	)
	.unwrap();

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req/" + user_to_accept.user_id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(join)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let join_res: GroupAcceptJoinReqServerOutput = handle_server_response(body.as_str()).unwrap();
	assert_eq!(join_res.session_id, None);

	//user is already saved

	//______________________________________________________________________________________________
	//3. should get the group data
	let _data = get_group(
		secret_token,
		user.user_data.jwt.as_str(),
		group.group_id.as_str(),
		&user.user_data.user_keys[0].private_key,
		false,
	)
	.await;
}

//__________________________________________________________________________________________________
//key rotation

#[tokio::test]
async fn test_31_start_key_rotation()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let mut group = GROUP_TEST_STATE.get().unwrap().write().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[0];

	let pre_group_key = &group
		.decrypted_group_keys
		.get(user.user_id.as_str())
		.unwrap()[0]
		.group_key;
	let invoker_public_key = &user.user_data.user_keys[0].public_key;

	let input = TestGroup::key_rotation(pre_group_key, invoker_public_key, false, None, "test".to_string()).unwrap();

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/key_rotation");
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out = ServerOutput::<KeyRotationStartServerOutput>::from_string(body.as_str()).unwrap();

	assert!(out.status);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	//______________________________________________________________________________________________
	//now get the new key, no need for done key rotation because the invoker is already done

	let data_user_0 = get_group(
		secret_token,
		user.user_data.jwt.as_str(),
		out.group_id.as_str(),
		&user.user_data.user_keys[0].private_key,
		false,
	)
	.await;

	assert_eq!(data_user_0.1.len(), 2);

	group
		.decrypted_group_keys
		.insert(user.user_id.to_string(), data_user_0.1);

	//wait a bit to finish the key rotation in the sub thread
	tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_32_done_key_rotation_for_other_user()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let mut group = GROUP_TEST_STATE.get().unwrap().write().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;

	let user = &users[1];

	//should not have the new group key before done key rotation
	let data_user = get_group(
		secret_token,
		user.user_data.jwt.as_str(),
		group.group_id.as_str(),
		&user.user_data.user_keys[0].private_key,
		true,
	)
	.await;

	//still one key
	assert_eq!(data_user.1.len(), 1);
	assert!(data_user.0.key_update); //notify the user that there is a key update

	//get the data for the rotation

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/key_rotation");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out = ServerOutput::<Vec<sentc_crypto::sdk_common::group::KeyRotationInput>>::from_string(body.as_str()).unwrap();

	assert!(out.status);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	//only one new key, not for each user key!
	assert_eq!(out.len(), 1);

	let newest_key_id_to_fetch = out[0].new_group_key_id.clone();

	//done it for each key
	for key in out {
		let key_id = key.new_group_key_id.clone();

		let rotation_out = TestGroup::done_key_rotation(
			&user.user_data.user_keys[0].private_key,
			&user.user_data.user_keys[0].public_key,
			&group
				.decrypted_group_keys
				.get(user.user_id.as_str())
				.unwrap()[0]
				.group_key,
			key,
		)
		.unwrap();

		//done the key rotation to save the new key
		let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/key_rotation/" + key_id.as_str());
		let client = reqwest::Client::new();
		let res = client
			.put(url)
			.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
			.header("x-sentc-app-token", secret_token)
			.body(rotation_out)
			.send()
			.await
			.unwrap();

		let body = res.text().await.unwrap();
		handle_general_server_response(body.as_str()).unwrap();
	}

	let data_user_1 = get_group(
		secret_token,
		user.user_data.jwt.as_str(),
		group.group_id.as_str(),
		&user.user_data.user_keys[0].private_key,
		false,
	)
	.await;

	//now both keys must be there
	assert_eq!(data_user_1.1.len(), 2);

	group
		.decrypted_group_keys
		.insert(user.user_id.to_string(), data_user_1.1);

	//get the key via direct fetch
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/key/" + newest_key_id_to_fetch.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let new_key = sentc_crypto::group::get_group_key_from_server_output(body.as_str()).unwrap();

	let _decrypted_key = TestGroup::decrypt_group_keys(&user.user_data.user_keys[0].private_key, new_key, None).unwrap();
}

#[tokio::test]
async fn test_33_get_key_with_pagination()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[0];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/keys/0/abc");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out = ServerOutput::<Vec<sentc_crypto::sdk_common::group::GroupKeyServerOutput>>::from_string(body.as_str()).unwrap();
	assert!(out.status);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();
	assert_eq!(out.len(), 2);

	let group_keys_fetch = sentc_crypto::group::get_group_keys_from_server_output(body.as_str()).unwrap();

	let mut group_keys = Vec::with_capacity(group_keys_fetch.len());

	for group_keys_fetch in group_keys_fetch {
		group_keys.push(TestGroup::decrypt_group_keys(&user.user_data.user_keys[0].private_key, group_keys_fetch, None).unwrap());
	}

	//normally use len() - 1 but this time we won't fake pagination, so we don't use the last item
	let latest_fetched_id = group_keys[group_keys.len() - 2].group_key.key_id.as_str();
	let last_fetched_time = group_keys[group_keys.len() - 2].time;

	//fetch it with pagination (a fake page two)
	let url =
		get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/keys/" + last_fetched_time.to_string().as_str() + "/" + latest_fetched_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out = ServerOutput::<Vec<sentc_crypto::sdk_common::group::GroupKeyServerOutput>>::from_string(body.as_str()).unwrap();
	assert!(out.status);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();
	assert_eq!(out.len(), 1);

	assert_ne!(out[0].group_key_id.to_string(), latest_fetched_id.to_string())
}

#[tokio::test]
async fn test_34_invite_user_with_two_keys()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let mut group = GROUP_TEST_STATE.get().unwrap().write().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];

	let user_to_invite = &users[2];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	let user_group_data_2 = add_user_by_invite(
		secret_token,
		creator.user_data.jwt.as_str(),
		group.group_id.as_str(),
		user_keys,
		user_to_invite.user_id.as_str(),
		user_to_invite.user_data.jwt.as_str(),
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&user_to_invite.user_data.user_keys[0].private_key,
	)
	.await;

	//should get all keys
	assert_eq!(user_group_data_2.1.len(), 2);

	group
		.decrypted_group_keys
		.insert(user_to_invite.user_id.to_string(), user_group_data_2.1);
}

//__________________________________________________________________________________________________

#[tokio::test]
async fn test_35_update_rank()
{
	//update the rank of a user and check if the rank for the child group is also updated

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user_to_change = &users[1];

	let input = GroupChangeRankServerInput {
		changed_user_id: user_to_change.user_id.to_string(),
		new_rank: 2,
	};

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/change_rank");
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();

	//get the group data with the new rank
	let data = get_group(
		secret_token,
		user_to_change.user_data.jwt.as_str(),
		group.group_id.as_str(),
		&user_to_change.user_data.user_keys[0].private_key,
		false,
	)
	.await;

	assert_eq!(data.0.rank, 2);
}

#[tokio::test]
async fn test_36_no_rank_change_without_permission()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user_without_permission = &users[1];

	let input = GroupChangeRankServerInput {
		changed_user_id: user_without_permission.user_id.to_string(),
		new_rank: 2,
	};

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/change_rank");
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(
			AUTHORIZATION,
			auth_header(user_without_permission.user_data.jwt.as_str()),
		)
		.header("x-sentc-app-token", secret_token)
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out = ServerOutput::<ServerSuccessOutput>::from_string(body.as_str()).unwrap();

	assert!(!out.status);
}

#[tokio::test]
async fn test_37_get_group_user()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user_2 = &users[2];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/member/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let res: Vec<GroupUserListItem> = handle_server_response(body.as_str()).unwrap();

	assert_eq!(res.len(), 2);

	//the caller id should not be in this arr
	assert_ne!(res[0].user_id, creator.user_id);
	assert_ne!(res[1].user_id, creator.user_id);

	//______________________________________________________________________________________________
	//should get the 2nd page

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];

	let url = get_url(
		"api/v1/group/".to_owned() + group.group_id.as_str() + "/member/" + res[0].joined_time.to_string().as_str() + "/" + res[0].user_id.as_str(),
	);

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let res: Vec<GroupUserListItem> = handle_server_response(body.as_str()).unwrap();

	assert_eq!(res.len(), 1);
	assert_ne!(res[0].user_id, creator.user_id);

	//______________________________________________________________________________________________

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/member/0/none");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(user_2.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let res: Vec<GroupUserListItem> = handle_server_response(body.as_str()).unwrap();

	assert_eq!(res.len(), 2);

	//the caller id should not be in this arr
	assert_ne!(res[0].user_id, user_2.user_id);
	assert_ne!(res[1].user_id, user_2.user_id);
}

#[tokio::test]
async fn test_38_kick_user_from_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user_to_kick = &users[2];

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/kick/" + &user_to_kick.user_id);

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
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
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<ServerSuccessOutput>::from_string(body.as_str()).unwrap();

	assert!(!out.status);
}

#[tokio::test]
async fn test_39_key_rotation_in_child_group_by_parent()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;
	let mut child_group = CHILD_GROUP_TEST_STATE.get().unwrap().write().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user_for_child_group = &users[3];

	//______________________________________________________________________________________________
	//invite the user to the child group

	let user_keys = child_group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	let user_group_data_2 = add_user_by_invite(
		secret_token,
		creator.user_data.jwt.as_str(),
		child_group.group_id.as_str(),
		user_keys,
		user_for_child_group.user_id.as_str(),
		user_for_child_group.user_data.jwt.as_str(),
		&user_for_child_group.user_data.user_keys[0].exported_public_key,
		&user_for_child_group.user_data.user_keys[0].private_key,
	)
	.await;

	child_group
		.decrypted_group_keys
		.insert(user_for_child_group.user_id.to_string(), user_group_data_2.1);

	//______________________________________________________________________________________________
	//do key rotation in the child group, triggered by the parent group

	let pre_group_key = &child_group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap()[0]
		.group_key;

	//get the right key for the child group from index 1 because the key rotation (in the parent) changes the index
	let group_public_key = &group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap()[1]
		.public_group_key;
	let group_private_key = &group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap()[1]
		.private_group_key;

	let new_group_data = key_rotation(
		secret_token,
		creator.user_data.jwt.as_str(),
		child_group.group_id.as_str(),
		pre_group_key,
		group_public_key,
		group_private_key,
		None,
	)
	.await;

	//______________________________________________________________________________________________
	//done key rotation for the direct group member

	let new_group_data_for_new_user = done_key_rotation(
		secret_token,
		user_for_child_group.user_data.jwt.as_str(),
		child_group.group_id.as_str(),
		pre_group_key,
		&user_for_child_group.user_data.user_keys[0].public_key,
		&user_for_child_group.user_data.user_keys[0].private_key,
		None,
	)
	.await;

	child_group
		.decrypted_group_keys
		.insert(creator.user_id.to_string(), new_group_data.1);

	let keys = child_group
		.decrypted_group_keys
		.get_mut(&user_for_child_group.user_id)
		.unwrap();

	for key in new_group_data_for_new_user {
		keys.push(key)
	}
}

#[tokio::test]
async fn test_40_key_rotation_in_child_group_by_user()
{
	//this time start the key rotation from a group user and check if the parent got the keys too

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;
	let mut child_group = CHILD_GROUP_TEST_STATE.get().unwrap().write().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user_for_child_group = &users[3];

	//this should be the newest key because the keys are ordered by time
	let pre_group_key = &child_group
		.decrypted_group_keys
		.get(user_for_child_group.user_id.as_str())
		.unwrap()[0]
		.group_key;

	//the newest group public key was used to do the key rotation
	let group_public_key = &group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap()[0]
		.public_group_key;
	let group_private_key = &group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap()[0]
		.private_group_key;

	let new_group_data = key_rotation(
		secret_token,
		user_for_child_group.user_data.jwt.as_str(),
		child_group.group_id.as_str(),
		pre_group_key,
		&user_for_child_group.user_data.user_keys[0].public_key,
		&user_for_child_group.user_data.user_keys[0].private_key,
		Default::default(),
	)
	.await;

	//______________________________________________________________________________________________
	//done key rotation for the parent group member, should use the newest group keys for key rotation
	let new_group_data_for_parent = done_key_rotation(
		secret_token,
		creator.user_data.jwt.as_str(),
		child_group.group_id.as_str(),
		pre_group_key,
		group_public_key,
		group_private_key,
		None,
	)
	.await;

	child_group
		.decrypted_group_keys
		.insert(creator.user_id.to_string(), new_group_data.1);

	let keys = child_group
		.decrypted_group_keys
		.get_mut(&user_for_child_group.user_id)
		.unwrap();

	for key in new_group_data_for_parent {
		keys.push(key)
	}
}

#[tokio::test]
async fn test_41_key_rotation_with_multiple_user_keys()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let mut group = GROUP_TEST_STATE.get().unwrap().write().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;

	let user = &users[0];
	let user_1 = &users[1];

	let jwt = &user_1.user_data.jwt;
	let pre_group_key = &user_1.user_data.user_keys[0].group_key;
	let device_invoker_public_key = &user_1.user_data.device_keys.public_key;
	let device_invoker_private_key = &user_1.user_data.device_keys.private_key;

	//first do a key rotation in a user group to test if only the latest key of a user will be used

	user_key_rotation(
		secret_token,
		jwt,
		pre_group_key,
		device_invoker_public_key,
		device_invoker_private_key,
	)
	.await;

	//then do a key rotation in the group

	let pre_group_key = &group
		.decrypted_group_keys
		.get(user.user_id.as_str())
		.unwrap()[1]
		.group_key;

	let new_group_data = key_rotation(
		secret_token,
		user.user_data.jwt.as_str(),
		group.group_id.as_str(),
		pre_group_key,
		&user.user_data.user_keys[0].public_key,
		&user.user_data.user_keys[0].private_key,
		None,
	)
	.await;

	group
		.decrypted_group_keys
		.insert(user.user_id.to_string(), new_group_data.1);

	//now get the key rotation for another user

	//check if the user needs a rotation
	let _data_user = get_group(
		secret_token,
		jwt,
		group.group_id.as_str(),
		&user_1.user_data.user_keys[0].private_key,
		true,
	)
	.await;

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/key_rotation");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<sentc_crypto::sdk_common::group::KeyRotationInput> = handle_server_response(body.as_str()).unwrap();

	//only one key here, even when the user got multiple keys in his/her user group
	assert_eq!(out.len(), 1);

	//no need to do the rotation. this test is all about the right key count
}

//__________________________________________________________________________________________________
//delete group

#[tokio::test]
async fn test_42_delete_group()
{
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();
}

#[tokio::test]
async fn test_43_get_all_groups_to_user()
{
	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	//create more groups for pagination
	let group_1 = create_group(
		secret_token,
		&creator.user_data.user_keys[0].public_key,
		None,
		creator.user_data.jwt.as_str(),
	)
	.await;

	let group_2 = create_group(
		secret_token,
		&creator.user_data.user_keys[0].public_key,
		None,
		creator.user_data.jwt.as_str(),
	)
	.await;

	let group_3 = create_group(
		secret_token,
		&creator.user_data.user_keys[0].public_key,
		None,
		creator.user_data.jwt.as_str(),
	)
	.await;

	let url = get_url("api/v1/group/all".to_owned() + "/0/none");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<Vec<ListGroups>>::from_string(body.as_str()).unwrap();

	assert!(out.status);

	let out = out.result.unwrap();

	assert_eq!(out.len(), 3);

	//Order by time
	assert_eq!(out[0].group_id, group_1);
	assert_eq!(out[1].group_id, group_2);
	assert_eq!(out[2].group_id, group_3);

	//test pagination

	let url = get_url("api/v1/group/all".to_owned() + "/" + out[0].time.to_string().as_str() + "/" + out[0].group_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: Vec<ListGroups> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 2);

	assert_eq!(out[0].group_id, group_2);
	assert_eq!(out[1].group_id, group_3);

	//test groups are deleted by the cleanup fn
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
		delete_user(secret_token, user.username.clone()).await;
	}

	let customer_jwt = &CUSTOMER_TEST_STATE.get().unwrap().read().await.verify.jwt;

	delete_app(customer_jwt, app.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

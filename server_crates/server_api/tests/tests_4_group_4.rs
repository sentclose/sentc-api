use std::collections::HashMap;

use reqwest::header::AUTHORIZATION;
use sentc_crypto::group::GroupKeyData;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::{SdkError, UserData};
use sentc_crypto_common::group::{GroupAcceptJoinReqServerOutput, GroupInviteServerOutput};
use sentc_crypto_common::{GroupId, UserId};
use server_api_common::app::AppRegisterOutput;
use server_api_common::customer::CustomerDoneLoginOutput;
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

	for i in 0..3 {
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

#[tokio::test]
async fn test_10_create_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let group_id = create_group(
		secret_token.as_str(),
		&creator.user_data.user_keys[0].public_key,
		None,
		&creator.user_data.jwt,
	)
	.await;

	let mut groups = Vec::with_capacity(1);

	let (_data, group_data_for_creator) = get_group(
		secret_token.as_str(),
		&creator.user_data.jwt,
		group_id.as_str(),
		&creator.user_data.user_keys[0].private_key,
		false,
	)
	.await;

	let mut decrypted_group_keys = HashMap::new();

	decrypted_group_keys.insert(creator.user_id.to_string(), group_data_for_creator);

	groups.push(GroupState {
		group_id,
		group_member: vec![creator.user_id.to_string()],
		decrypted_group_keys,
	});

	GROUP_TEST_STATE
		.get_or_init(|| async move { RwLock::new(groups) })
		.await;
}

#[tokio::test]
async fn test_11_not_invite_user_with_wrong_rank()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = &GROUP_TEST_STATE.get().unwrap().read().await[0];

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

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&group_keys_ref,
		false,
		Some(0),
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
		Ok(_) => panic!("must be error"),
		Err(e) => {
			match e {
				SdkError::ServerErr(code, _) => {
					assert_eq!(code, 301);
				},
				_ => panic!("should be server error"),
			}
		},
	}
}

#[tokio::test]
async fn test_12_invite_user_with_rank()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let groups = &mut GROUP_TEST_STATE.get().unwrap().write().await;
	let group = &mut groups[0];

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

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&group_keys_ref,
		false,
		Some(2),
	)
	.unwrap();

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/invite_auto/" + user_to_invite.user_id.as_str());

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

	let (data, group_data_for_creator) = get_group(
		secret_token.as_str(),
		&user_to_invite.user_data.jwt,
		group.group_id.as_str(),
		&user_to_invite.user_data.user_keys[0].private_key,
		false,
	)
	.await;

	assert_eq!(data.rank, 2);

	group
		.decrypted_group_keys
		.insert(user_to_invite.user_id.clone(), group_data_for_creator);
}

#[tokio::test]
async fn test_13_not_invite_user_with_higher_rank()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = &GROUP_TEST_STATE.get().unwrap().read().await[0];

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[1];

	let user_to_invite = &users[2];

	let mut group_keys_ref = vec![];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&group_keys_ref,
		false,
		Some(1), //user is rank 2 and cannot increase the rank
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
		Ok(_) => panic!("must be error"),
		Err(e) => {
			match e {
				SdkError::ServerErr(code, _) => {
					assert_eq!(code, 301);
				},
				_ => panic!("should be server error"),
			}
		},
	}
}

#[tokio::test]
async fn test_14_invite_user_from_another_user_with_rank()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = &GROUP_TEST_STATE.get().unwrap().read().await[0];

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[1];

	let user_to_invite = &users[2];

	let mut group_keys_ref = vec![];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&group_keys_ref,
		false,
		Some(2), //user is rank 2 and cannot increase the rank
	)
	.unwrap();

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/invite_auto/" + user_to_invite.user_id.as_str());

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

	let (data, _group_data_for_creator) = get_group(
		secret_token.as_str(),
		&user_to_invite.user_data.jwt,
		group.group_id.as_str(),
		&user_to_invite.user_data.user_keys[0].private_key,
		false,
	)
	.await;

	assert_eq!(data.rank, 2);

	//kick the user from the group for other tests
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/kick/" + &users[2].user_id);

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(&users[0].user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	handle_general_server_response(body.as_str()).unwrap();
}

//__________________________________________________________________________________________________
//join req test

#[tokio::test]
async fn test_15_not_accept_join_with_wrong_rank()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = &GROUP_TEST_STATE.get().unwrap().read().await[0];

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];

	let user_to_invite = &users[2];

	//send join req
	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req");
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

	//______________________________________________________________________________________________
	//accept the join req

	let mut group_keys_ref = vec![];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&group_keys_ref,
		false,
		Some(0),
	)
	.unwrap();

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req/" + user_to_invite.user_id.as_str());

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

	match handle_server_response::<GroupAcceptJoinReqServerOutput>(body.as_str()) {
		Ok(_) => panic!("must be error"),
		Err(e) => {
			match e {
				SdkError::ServerErr(code, _) => {
					assert_eq!(code, 301);
				},
				_ => panic!("should be server error"),
			}
		},
	}
}

#[tokio::test]
async fn test_16_not_accept_join_with_higher_rank()
{
	//not send join req again

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = &GROUP_TEST_STATE.get().unwrap().read().await[0];

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[1];

	let user_to_invite = &users[2];

	let mut group_keys_ref = vec![];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&group_keys_ref,
		false,
		Some(1),
	)
	.unwrap();

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req/" + user_to_invite.user_id.as_str());

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

	match handle_server_response::<GroupAcceptJoinReqServerOutput>(body.as_str()) {
		Ok(_) => panic!("must be error"),
		Err(e) => {
			match e {
				SdkError::ServerErr(code, _) => {
					assert_eq!(code, 301);
				},
				_ => panic!("should be server error"),
			}
		},
	}
}

#[tokio::test]
async fn test_16_accept_join_with_rank()
{
	//not send join req again

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = &GROUP_TEST_STATE.get().unwrap().read().await[0];

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[1];

	let user_to_invite = &users[2];

	let mut group_keys_ref = vec![];

	let user_keys = group
		.decrypted_group_keys
		.get(creator.user_id.as_str())
		.unwrap();

	for decrypted_group_key in user_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(
		&user_to_invite.user_data.user_keys[0].exported_public_key,
		&group_keys_ref,
		false,
		Some(2),
	)
	.unwrap();

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str() + "/join_req/" + user_to_invite.user_id.as_str());

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

	let join_res: GroupAcceptJoinReqServerOutput = handle_server_response(body.as_str()).unwrap();
	assert_eq!(join_res.session_id, None);

	let (data, _) = get_group(
		secret_token,
		user_to_invite.user_data.jwt.as_str(),
		group.group_id.as_str(),
		&user_to_invite.user_data.user_keys[0].private_key,
		false,
	)
	.await;

	assert_eq!(data.rank, 2);
}

//__________________________________________________________________________________________________

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

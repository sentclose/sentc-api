use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use sentc_crypto::{KeyData, SymKeyFormat};
use sentc_crypto_common::group::{GroupCreateOutput, GroupDeleteServerOutput, GroupServerData};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::{GroupId, ServerOutput, UserId};
use server_api::AppRegisterOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{auth_header, create_app, create_group, create_test_user, delete_app, delete_user, get_url};

mod test_fn;

pub struct UserState
{
	pub username: String,
	pub pw: String,
	pub user_id: UserId,
	pub key_data: KeyData,
}

pub struct GroupState
{
	pub group_id: GroupId,
	pub group_member: Vec<UserId>,
	pub group_server_data: Vec<GroupServerData>,
	pub decrypted_group_keys: Vec<SymKeyFormat>,
}

static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<Vec<UserState>>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<GroupState>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_global_test()
{
	//create here an app
	let app_data = create_app().await;

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
			key_data,
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
					group_server_data: vec![],
					decrypted_group_keys: vec![],
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

	let group_input = sentc_crypto::group::prepare_create(&creator.key_data.public_key, None).unwrap();

	let url = get_url("api/v1/group".to_owned());
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(creator.key_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	let out = ServerOutput::<GroupCreateOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	group.group_id = out.group_id;
}

#[tokio::test]
async fn test_11_create_child_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let _child_id = create_group(
		secret_token,
		&creator.key_data.public_key,
		Some(group.group_id.to_string()),
		creator.key_data.jwt.as_str(),
	)
	.await;

	//TODO get group data

	//don't delete the child group to test if parent group delete deletes all. delete the child
}

#[ignore]
#[tokio::test]
async fn test_12_invite_user()
{
	//TODO did this test when get data is implemented, so we can get the decrypted group keys

	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];

	let user_to_invite = &users[1];

	let mut group_keys_ref = vec![];

	for decrypted_group_key in &group.decrypted_group_keys {
		group_keys_ref.push(decrypted_group_key);
	}

	let invite = sentc_crypto::group::prepare_group_keys_for_new_member(&user_to_invite.key_data.exported_public_key, &group_keys_ref).unwrap();

	let url = get_url("api/v1/group".to_owned() + group.group_id.as_str() + "/invite/" + user_to_invite.user_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(creator.key_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(invite)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out = ServerOutput::<ServerSuccessOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);
}

#[tokio::test]
async fn test_30_delete_group()
{
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let url = get_url("api/v1/group/".to_owned() + group.group_id.as_str());
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(creator.key_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	let out = ServerOutput::<GroupDeleteServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	assert_eq!(out.group_id, group.group_id);
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
		delete_user(secret_token, user.key_data.jwt.as_str(), user.user_id.as_str()).await;
	}

	delete_app(app.app_id.as_str()).await;
}

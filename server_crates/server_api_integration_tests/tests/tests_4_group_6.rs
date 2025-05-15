//Force group fn

use reqwest::header::AUTHORIZATION;
use sentc_crypto::sdk_common::group::GroupInviteServerOutput;
use sentc_crypto::sdk_utils::error::SdkUtilError;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::SdkError;
use sentc_crypto_common::group::{GroupCreateOutput, GroupLightServerData, GroupUserListItem};
use sentc_crypto_common::{GroupId, UserId};
use sentc_crypto_light::sdk_common::group::GroupNewMemberLightInput;
use sentc_crypto_light::UserDataInt as UserDataIntLight;
use server_dashboard_common::app::AppRegisterOutput;
use server_dashboard_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
	add_user_by_invite,
	auth_header,
	create_app,
	create_test_customer,
	create_test_user,
	customer_delete,
	delete_app,
	delete_user,
	get_base_url,
	get_group,
	get_group_from_group_as_member,
	get_url,
	login_user,
	TestGroup,
	TestGroupKeyData,
	TestUser,
	TestUserDataInt as UserDataIntFull,
};

mod test_fn;

pub struct UserState
{
	pub username: String,
	pub pw: String,
	pub user_id: UserId,
	pub user_data: UserDataIntFull,
}

pub struct GroupState
{
	pub group_id: GroupId,
	pub decrypted_group_keys: Vec<TestGroupKeyData>,
}

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerDoneLoginOutput>> = OnceCell::const_new();
static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<Vec<UserState>>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<GroupState>> = OnceCell::const_new();

pub struct UserStateLight
{
	pub username: String,
	pub pw: String,
	pub user_id: UserId,
	pub user_data: UserDataIntLight,
}

pub struct GroupStateLight
{
	pub group_id: GroupId,
}

static USERS_TEST_STATE_LIGHT: OnceCell<RwLock<Vec<UserStateLight>>> = OnceCell::const_new();
static GROUP_TEST_STATE_LIGHT: OnceCell<RwLock<GroupStateLight>> = OnceCell::const_new();

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

	for i in 0..2 {
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

	//light users

	let mut users = vec![];

	for i in 0..2 {
		let username = "hi1".to_string() + i.to_string().as_str();

		let user_id = sentc_crypto_light::util_req_full::user::register(get_base_url(), secret_token_str, &username, user_pw)
			.await
			.unwrap();

		let out = sentc_crypto_light::util_req_full::user::login(get_base_url(), public_token_str, &username, user_pw)
			.await
			.unwrap();

		let key_data = if let sentc_crypto_light::util_req_full::user::PreLoginOut::Direct(d) = out {
			d
		} else {
			panic!("No mfa excepted");
		};

		let user = UserStateLight {
			username,
			pw: user_pw.to_string(),
			user_id,
			user_data: key_data,
		};

		users.push(user);
	}

	USERS_TEST_STATE_LIGHT
		.get_or_init(|| async move { RwLock::new(users) })
		.await;
}

//__________________________________________________________________________________________________

#[tokio::test]
async fn test_10_create_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let group_input = TestGroup::prepare_create(&creator.user_data.user_keys[0].public_key, None, Default::default()).unwrap();

	let url = get_url("api/v1/group/forced/".to_owned() + &creator.user_id);
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &out_1.group_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let data = sentc_crypto::group::get_group_data(body.as_str()).unwrap();

	let mut decrypted_group_keys = Vec::with_capacity(data.keys.len());

	for key in data.keys {
		decrypted_group_keys.push(TestGroup::decrypt_group_keys(&creator.user_data.user_keys[0].private_key, key, None).unwrap());
	}

	GROUP_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(GroupState {
					group_id: out_1.group_id,
					decrypted_group_keys,
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_10_delete_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let mut group = GROUP_TEST_STATE.get().unwrap().write().await;
	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let client = reqwest::Client::new();
	let res = client
		.delete(get_url("api/v1/group/forced/".to_owned() + &group.group_id))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//fetch the group
	let url = get_url("api/v1/group/".to_owned() + &group.group_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	match sentc_crypto::group::get_group_data(body.as_str()) {
		Err(SdkError::Util(SdkUtilError::ServerErr(s, _))) => {
			assert_eq!(s, 310);
		},
		_ => {
			panic!("Must be error")
		},
	}

	//now create the group again for the other tests
	let group_input = TestGroup::prepare_create(&creator.user_data.user_keys[0].public_key, None, Default::default()).unwrap();

	let url = get_url("api/v1/group/forced/".to_owned() + &creator.user_id);
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &out_1.group_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let data = sentc_crypto::group::get_group_data(body.as_str()).unwrap();

	let mut decrypted_group_keys = Vec::with_capacity(data.keys.len());

	for key in data.keys {
		decrypted_group_keys.push(TestGroup::decrypt_group_keys(&creator.user_data.user_keys[0].private_key, key, None).unwrap());
	}

	group.group_id = out_1.group_id;
	group.decrypted_group_keys = decrypted_group_keys;
}

#[tokio::test]
async fn test_11_create_light_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let creator = USERS_TEST_STATE_LIGHT.get().unwrap().read().await;
	let creator = &creator[0];

	let client = reqwest::Client::new();
	let res = client
		.post(get_url(
			"api/v1/group/forced/".to_owned() + &creator.user_id + "/light",
		))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &out_1.group_id + "/light");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let _out: GroupLightServerData = handle_server_response(&body).unwrap();

	GROUP_TEST_STATE_LIGHT
		.get_or_init(|| {
			async move {
				RwLock::new(GroupStateLight {
					group_id: out_1.group_id,
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_13_create_child_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	//use here the public group key for child group!
	let group_public_key = &group.decrypted_group_keys[0].public_group_key;
	let group_private_key = &group.decrypted_group_keys[0].private_group_key;

	let group_input = TestGroup::prepare_create(group_public_key, None, Default::default()).unwrap();

	let url = get_url("api/v1/group/forced/".to_owned() + &creator.user_id + "/" + &group.group_id + "/child");
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let (_data, _keys) = get_group(
		secret_token,
		creator.user_data.jwt.as_str(),
		&out_1.group_id,
		group_private_key,
		false,
	)
	.await;
}

#[tokio::test]
async fn test_14_create_child_group_light()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE_LIGHT.get().unwrap().read().await;

	let creator = USERS_TEST_STATE_LIGHT.get().unwrap().read().await;
	let creator = &creator[0];

	let client = reqwest::Client::new();
	let res = client
		.post(get_url(
			"api/v1/group/forced/".to_owned() + &creator.user_id + "/" + &group.group_id + "/child/light",
		))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &out_1.group_id + "/light");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let _out: GroupLightServerData = handle_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_15_create_connected_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	//use here the public group key for child group!
	let group_public_key = &group.decrypted_group_keys[0].public_group_key;
	let group_private_key = &group.decrypted_group_keys[0].private_group_key;

	let group_input = TestGroup::prepare_create(group_public_key, None, Default::default()).unwrap();

	let url = get_url("api/v1/group/forced/".to_owned() + &creator.user_id + "/" + &group.group_id + "/connected");
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.body(group_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let (_data, _keys) = get_group_from_group_as_member(
		secret_token,
		creator.user_data.jwt.as_str(),
		&out_1.group_id,
		&group.group_id,
		group_private_key,
	)
	.await;
}

#[tokio::test]
async fn test_16_create_connected_group_light()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE_LIGHT.get().unwrap().read().await;

	let creator = USERS_TEST_STATE_LIGHT.get().unwrap().read().await;
	let creator = &creator[0];

	let url = get_url("api/v1/group/forced/".to_owned() + &creator.user_id + "/" + &group.group_id + "/connected/light");
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out_1: GroupCreateOutput = handle_server_response(&body).unwrap();

	let url = get_url("api/v1/group/".to_owned() + &out_1.group_id + "/light");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.header("x-sentc-group-access-id", &group.group_id)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let _out: GroupLightServerData = handle_server_response(&body).unwrap();
}

//user invite

#[tokio::test]
async fn test_17_force_user_invite()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user = &users[1];

	let mut group_keys_ref = vec![];

	for decrypted_group_key in &group.decrypted_group_keys {
		group_keys_ref.push(&decrypted_group_key.group_key);
	}

	let data = TestGroup::prepare_group_keys_for_new_member(
		&user
			.user_data
			.user_keys
			.first()
			.unwrap()
			.exported_public_key,
		&group_keys_ref,
		false,
		None,
	)
	.unwrap();

	let url = get_url(format!(
		"api/v1/group/forced/{}/{}/invite_auto/{}",
		&creator.user_id, &group.group_id, &user.user_id
	));

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.body(data)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	let out: GroupInviteServerOutput = handle_server_response(&body).unwrap();

	assert_eq!(out.session_id, None);

	let (data, _keys) = get_group(
		secret_token,
		&user.user_data.jwt,
		&group.group_id,
		&user.user_data.user_keys.first().unwrap().private_key,
		false,
	)
	.await;

	assert_eq!(data.rank, 4);
}

#[tokio::test]
async fn test_17_z_force_check_user_in_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	//let creator = &users[0];
	let user = &users[1];

	let url = get_url(format!(
		"api/v1/group/forced/{}/user/{}",
		&group.group_id, &user.user_id
	));

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	let out: GroupUserListItem = handle_server_response(&body).unwrap();

	assert_eq!(out.user_id, user.user_id);
}

#[tokio::test]
async fn test_18_force_kick_user()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user = &users[1];

	let url = get_url(format!(
		"api/v1/group/forced/{}/{}/kick/{}",
		&creator.user_id, &group.group_id, &user.user_id
	));

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();
	handle_general_server_response(&body).unwrap();

	//should not be able to fetch the group
	let data = sentc_crypto::util_req_full::group::get_group(
		get_base_url(),
		secret_token,
		&user.user_data.jwt,
		&group.group_id,
		None,
	)
	.await;

	assert!(data.is_err());
}

#[tokio::test]
async fn test_18_z_force_check_user_in_group_after_kick()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	//let creator = &users[0];
	let user = &users[1];

	let url = get_url(format!(
		"api/v1/group/forced/{}/user/{}",
		&group.group_id, &user.user_id
	));

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	let out: Result<GroupUserListItem, SdkError> = handle_server_response(&body);

	assert!(out.is_err());
}

#[tokio::test]
async fn test_19_force_user_invite_light()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE_LIGHT.get().unwrap().read().await;

	let users = USERS_TEST_STATE_LIGHT.get().unwrap().read().await;
	let creator = &users[0];
	let user = &users[1];

	let url = get_url(format!(
		"api/v1/group/forced/{}/{}/invite_auto/{}/light",
		&creator.user_id, &group.group_id, &user.user_id
	));

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", secret_token)
		.body(
			serde_json::to_string(&GroupNewMemberLightInput {
				rank: None,
			})
			.unwrap(),
		)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	let data = sentc_crypto_light::util_req_full::group::get_group_light(
		get_base_url(),
		secret_token,
		&user.user_data.jwt,
		&group.group_id,
		None,
	)
	.await
	.unwrap();

	assert_eq!(data.rank, 4);
}

#[tokio::test]
async fn test_20_force_user_group_kick_light()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;

	let group = GROUP_TEST_STATE_LIGHT.get().unwrap().read().await;

	let users = USERS_TEST_STATE_LIGHT.get().unwrap().read().await;
	let creator = &users[0];
	let user = &users[1];

	let url = get_url(format!(
		"api/v1/group/forced/{}/{}/kick/{}",
		&creator.user_id, &group.group_id, &user.user_id
	));

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();
	handle_general_server_response(&body).unwrap();

	let data = sentc_crypto_light::util_req_full::group::get_group_light(
		get_base_url(),
		secret_token,
		&user.user_data.jwt,
		&group.group_id,
		None,
	)
	.await;

	assert!(data.is_err());
}

//__________________________________________________________________________________________________
//user reset test here to test what will happen if the user is in a group
#[tokio::test]
async fn test_user_force_reset()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user = &users[1];

	//1. invite the 2nd user to a group
	add_user_by_invite(
		secret_token,
		&creator.user_data.jwt,
		&group.group_id,
		&group.decrypted_group_keys,
		&user.user_id,
		&user.user_data.jwt,
		&user
			.user_data
			.user_keys
			.first()
			.unwrap()
			.exported_public_key,
		&user.user_data.user_keys.first().unwrap().private_key,
	)
	.await;

	//2. reset the 2nd user
	let input = TestUser::register(&user.username, "123456789").unwrap();

	let url = get_url("api/v1/user/forced/reset_user".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", secret_token)
		.body(input)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//3. test login with new user pw

	//not login with old pw
	let err = TestUser::login(get_base_url(), secret_token, &user.username, &user.pw).await;

	if let Err(SdkError::Util(SdkUtilError::ServerErr(e, _))) = err {
		assert_eq!(e, 112);
	} else {
		panic!("Should be an error");
	}

	let out = login_user(secret_token, &user.username, "123456789").await;

	//must be the same user id
	assert_eq!(out.user_id, user.user_id);

	//4. test group fetch (should be error)

	let data = sentc_crypto::util_req_full::group::get_group(get_base_url(), secret_token, &out.jwt, &group.group_id, None)
		.await
		.unwrap();

	//now decrypting keys should fail because the user has got new keys

	for key in data.keys {
		let error = TestGroup::decrypt_group_keys(&out.user_keys.first().unwrap().private_key, key, None);

		if let Err(_e) = error {
		} else {
			panic!("Must be decryption error");
		}
	}

	//5. re invite user to group
	let join_res = TestGroup::invite_user(
		get_base_url(),
		secret_token,
		&creator.user_data.jwt,
		&group.group_id,
		&out.user_id,
		1,
		None,
		1,
		false,
		false,
		true,
		&out.user_keys.first().unwrap().exported_public_key,
		&[&group.decrypted_group_keys.first().unwrap().group_key],
		None,
	)
	.await
	.unwrap();

	assert_eq!(join_res, None);

	//6. now fetch group as normal
	get_group(
		secret_token,
		&out.jwt,
		&group.group_id,
		&out.user_keys.first().unwrap().private_key,
		false,
	)
	.await;
}
//__________________________________________________________________________________________________

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

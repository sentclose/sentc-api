//This is about the group light

use hyper::header::AUTHORIZATION;
use reqwest::StatusCode;
use sentc_crypto::entities::user::UserDataInt;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto_common::group::{GroupCreateOutput, GroupInviteReqList, GroupLightServerData, GroupNewMemberLightInput, GroupServerData};
use sentc_crypto_common::{GroupId, UserId};
use serde_json::to_string;
use server_dashboard_common::app::AppRegisterOutput;
use server_dashboard_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{auth_header, create_app, create_test_customer, create_test_user, customer_delete, delete_app, delete_user, get_url};

mod test_fn;

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
	pub group_member: Vec<GroupLightServerData>,
}

static CUSTOMER_TEST_STATE: OnceCell<RwLock<CustomerDoneLoginOutput>> = OnceCell::const_new();
static APP_TEST_STATE: OnceCell<RwLock<AppRegisterOutput>> = OnceCell::const_new();
static USERS_TEST_STATE: OnceCell<RwLock<Vec<UserState>>> = OnceCell::const_new();
static GROUP_TEST_STATE: OnceCell<RwLock<Vec<GroupState>>> = OnceCell::const_new();

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

	let client = reqwest::Client::new();
	let res = client
		.post(get_url("api/v1/group/light".to_owned()))
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	let out: GroupCreateOutput = handle_server_response(&body).unwrap();

	GROUP_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(vec![GroupState {
					group_id: out.group_id,
					group_member: vec![],
				}])
			}
		})
		.await;
}

#[tokio::test]
async fn test_11_get_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let mut groups = GROUP_TEST_STATE.get().unwrap().write().await;

	let creator = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	//try to get the group normal
	let url = get_url("api/v1/group/".to_owned() + &groups[0].group_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: GroupServerData = handle_server_response(&body).unwrap();

	assert_eq!(out.rank, 0);
	assert!(out.keys.is_empty());
	assert!(out.hmac_keys.is_empty());

	//now get the light data
	let url = get_url("api/v1/group/".to_owned() + &groups[0].group_id + "/light");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&creator.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: GroupLightServerData = handle_server_response(&body).unwrap();

	assert_eq!(out.rank, 0);

	groups[0].group_member = vec![out];
}

#[tokio::test]
async fn test_12_invite_user_to_group()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user_to_invite = &users[1];

	let url = get_url("api/v1/group/".to_owned() + groups[0].group_id.as_str() + "/invite/" + user_to_invite.user_id.as_str() + "/light");

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(
			to_string(&GroupNewMemberLightInput {
				rank: None,
			})
			.unwrap(),
		)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_13_get_invite_for_user()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let group = &GROUP_TEST_STATE.get().unwrap().read().await[0];

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
	let out: Vec<GroupInviteReqList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].group_id.to_string(), group.group_id.to_string());
}

#[tokio::test]
async fn test_14_accept_invite()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let mut groups = GROUP_TEST_STATE.get().unwrap().write().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;

	let user_to_invite = &users[1];

	let url = get_url("api/v1/group/".to_owned() + groups[0].group_id.as_str() + "/invite");

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

	//test access
	let url = get_url("api/v1/group/".to_owned() + &groups[0].group_id + "/light");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&user_to_invite.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: GroupLightServerData = handle_server_response(&body).unwrap();

	assert_eq!(out.rank, 4);

	groups[0].group_member.push(out);
}

#[tokio::test]
async fn test_15_join_req()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let groups = GROUP_TEST_STATE.get().unwrap().read().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let user = &users[2];

	let url = get_url("api/v1/group/".to_owned() + groups[0].group_id.as_str() + "/join_req");
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
async fn test_16_accept_join_req()
{
	let secret_token = &APP_TEST_STATE.get().unwrap().read().await.secret_token;
	let mut groups = GROUP_TEST_STATE.get().unwrap().write().await;

	let users = USERS_TEST_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user_to_accept = &users[2];

	let url = get_url("api/v1/group/".to_owned() + groups[0].group_id.as_str() + "/join_req/" + user_to_accept.user_id.as_str() + "/light");

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(creator.user_data.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.body(
			to_string(&GroupNewMemberLightInput {
				rank: None,
			})
			.unwrap(),
		)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();

	//test access
	let url = get_url("api/v1/group/".to_owned() + &groups[0].group_id + "/light");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&user_to_accept.user_data.jwt))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: GroupLightServerData = handle_server_response(&body).unwrap();

	assert_eq!(out.rank, 4);

	groups[0].group_member.push(out);
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

	let customer_jwt = &CUSTOMER_TEST_STATE.get().unwrap().read().await.verify.jwt;

	delete_app(customer_jwt, app.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

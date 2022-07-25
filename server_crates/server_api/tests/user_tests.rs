use reqwest::StatusCode;
use sentc_crypto_common::user::{
	RegisterServerOutput,
	UserDeleteServerOutput,
	UserIdentifierAvailableServerInput,
	UserIdentifierAvailableServerOutput,
};
use sentc_crypto_common::ServerOutput;
use server_api::core::api_res::ApiErrorCodes;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::get_url;

mod test_fn;

pub struct UserState
{
	pub username: String,
	pub pw: String,
	pub user_id: String,
}

static USER_TEST_STATE: OnceCell<RwLock<UserState>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_global_test()
{
	//this fn must be execute first!
	USER_TEST_STATE
		.get_or_init(|| {
			async {
				RwLock::new(UserState {
					username: "admin_test".to_string(),
					pw: "12345".to_string(),
					user_id: "".to_string(),
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_1_user_exists()
{
	let username = &USER_TEST_STATE.get().unwrap().read().await.username;

	//test if user exists
	let input = UserIdentifierAvailableServerInput {
		user_identifier: username.to_owned(),
	};

	let url = get_url("api/v1/exists".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let exists = ServerOutput::<UserIdentifierAvailableServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(exists.status, true);
	assert_eq!(exists.err_code, None);

	let exists = match exists.result {
		Some(v) => v,
		None => panic!("exists is not here"),
	};

	assert_eq!(exists.user_identifier, username.to_string());
	assert_eq!(exists.available, false);
}

#[tokio::test]
async fn test_2_user_register()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;

	let username = &user.username;
	let pw = &user.pw;

	let url = get_url("api/v1/register".to_owned());

	let input = sentc_crypto::user::register(username, pw).unwrap();

	let client = reqwest::Client::new();
	let res = client.post(url).body(input).send().await.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	//check it here (not like the client) to see if the server respond correctly
	let register_out = ServerOutput::<RegisterServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(register_out.status, true);
	assert_eq!(register_out.err_code, None);

	let register_out = register_out.result.unwrap();
	assert_eq!(register_out.user_identifier, username.to_string());

	//get the user id like the client
	let user_id = sentc_crypto::user::done_register(body.as_str()).unwrap();

	assert_ne!(user_id, "".to_owned());

	//save the user id
	user.user_id = user_id;
}

#[tokio::test]
async fn test_3_user_check_after_register()
{
	let username = &USER_TEST_STATE.get().unwrap().read().await.username;

	//test if user exists
	let input = UserIdentifierAvailableServerInput {
		user_identifier: username.to_string(),
	};

	let url = get_url("api/v1/exists".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let exists = ServerOutput::<UserIdentifierAvailableServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(exists.status, true);
	assert_eq!(exists.err_code, None);

	let exists = exists.result.unwrap();

	assert_eq!(exists.user_identifier, username.to_string());
	assert_eq!(exists.available, true);
}

#[tokio::test]
async fn test_4_user_register_failed_username_exists()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let username = &user.username;
	let pw = &user.pw;

	let url = get_url("api/v1/register".to_owned());

	let input = sentc_crypto::user::register(username, pw).unwrap();

	let client = reqwest::Client::new();
	let res = client.post(url).body(input).send().await.unwrap();

	assert_eq!(res.status(), StatusCode::BAD_REQUEST);

	let body = res.text().await.unwrap();
	let error = ServerOutput::<RegisterServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(error.status, false);
	assert_eq!(error.result.is_none(), true);
	assert_eq!(error.err_code.unwrap(), ApiErrorCodes::UserExists.get_int_code());
}

#[tokio::test]
async fn test_5_user_delete()
{
	let user_id = &USER_TEST_STATE.get().unwrap().read().await.user_id;

	let url = get_url("api/v1/user/".to_owned() + user_id);
	let client = reqwest::Client::new();
	let res = client.delete(url).send().await.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	let delete_output = ServerOutput::<UserDeleteServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(delete_output.status, true);
	assert_eq!(delete_output.err_code, None);

	let delete_output = delete_output.result.unwrap();
	assert_eq!(delete_output.user_id, user_id.to_string());
	assert_eq!(delete_output.msg, "User deleted");
}

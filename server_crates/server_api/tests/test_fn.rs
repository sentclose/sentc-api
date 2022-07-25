use reqwest::StatusCode;
use sentc_crypto_common::user::UserDeleteServerOutput;
use sentc_crypto_common::{ServerOutput, UserId};

pub fn get_url(path: String) -> String
{
	format!("http://127.0.0.1:{}/{}", 3002, path)
}

pub async fn register_user(username: &str, password: &str) -> UserId
{
	let url = get_url("api/v1/register".to_owned());

	let input = sentc_crypto::user::register(username, password).unwrap();

	let client = reqwest::Client::new();
	let res = client.post(url).body(input).send().await.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let user_id = sentc_crypto::user::done_register(body.as_str()).unwrap();

	assert_ne!(user_id, "".to_owned());

	user_id
}

pub async fn delete_user(user_id: &str)
{
	let url = get_url("api/v1/user/".to_owned() + user_id);
	let client = reqwest::Client::new();
	let res = client.delete(url).send().await.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	//TODO change this to sdk done delete
	let delete_output = ServerOutput::<UserDeleteServerOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(delete_output.status, true);
	assert_eq!(delete_output.err_code, None);

	let delete_output = delete_output.result.unwrap();
	assert_eq!(delete_output.user_id, user_id.to_string());
	assert_eq!(delete_output.msg, "User deleted");
}

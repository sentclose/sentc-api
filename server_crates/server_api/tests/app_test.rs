use reqwest::StatusCode;
use sentc_crypto_common::ServerOutput;
use server_api::{AppRegisterInput, AppRegisterOutput, AppTokenRenewOutput};
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{create_app, get_url};

mod test_fn;

pub struct AppState
{
	pub app_id: String,
	pub app_public_token: String,
	pub app_secret_token: String,
}

static APP_TEST_STATE: OnceCell<RwLock<AppState>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_global_test()
{
	APP_TEST_STATE
		.get_or_init(|| {
			async {
				RwLock::new(AppState {
					app_id: "".to_string(),
					app_public_token: "".to_string(),
					app_secret_token: "".to_string(),
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_1_create_app()
{
	let url = get_url("api/v1/customer/app".to_owned());

	let input = AppRegisterInput {
		identifier: Some("My app".to_string()),
	};

	//TODO add here the customer jwt when customer mod is done

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<AppRegisterOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = match out.result {
		Some(v) => v,
		None => panic!("out is not here"),
	};

	//save the app output, jwt data is not needed here, only when customer wants to verify or create own jwt
	let mut app = APP_TEST_STATE.get().unwrap().write().await;

	app.app_public_token = out.public_token;
	app.app_secret_token = out.secret_token;
	app.app_id = out.app_id;
}

#[tokio::test]
#[ignore]
async fn test_2_update_app()
{
	//TODO
}

#[tokio::test]
#[ignore]
async fn test_3_delete_app()
{
	//TODO
}

#[tokio::test]
async fn test_4_create_app_test_fn()
{
	create_app().await;

	//TODO delete app in test fn
}

#[tokio::test]
async fn test_5_renew_tokens()
{
	let app_data = create_app().await;

	//TODO add here the customer jwt when customer mod is done

	let url = get_url("api/v1/customer/app/".to_owned() + app_data.app_id.as_str() + "/token_renew");

	let client = reqwest::Client::new();
	let res = client.patch(url).send().await.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<AppTokenRenewOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = match out.result {
		Some(v) => v,
		None => panic!("out is not here"),
	};

	//must be new tokens
	assert_ne!(out.secret_token, app_data.secret_token);
	assert_ne!(out.public_token, app_data.public_token);
}

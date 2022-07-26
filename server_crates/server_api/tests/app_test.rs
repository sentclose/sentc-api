use reqwest::StatusCode;
use sentc_crypto_common::ServerOutput;
use server_api::{
	AppDeleteOutput,
	AppJwtRegisterOutput,
	AppRegisterInput,
	AppRegisterOutput,
	AppTokenRenewOutput,
	AppUpdateOutput,
	JwtKeyDeleteOutput,
};
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{create_app, delete_app, get_url};

mod test_fn;

pub struct AppState
{
	pub app_id: String,
	pub app_public_token: String,
	pub app_secret_token: String,
	pub jwt_data: Option<Vec<AppJwtRegisterOutput>>,
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
					jwt_data: None,
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
	app.jwt_data = Some(vec![out.jwt_data]);
}

#[tokio::test]
async fn test_2_update_app()
{
	//TODO add here the customer jwt when customer mod is done

	let app = APP_TEST_STATE.get().unwrap().read().await;

	let input = AppRegisterInput {
		identifier: Some("My app updated".to_string()),
	};

	let url = get_url("api/v1/customer/app/".to_owned() + app.app_id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<AppUpdateOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();
	assert_eq!(out.app_id, app.app_id.to_string());
	assert_eq!(out.msg, "App updated");
}

#[tokio::test]
async fn test_3_renew_tokens()
{
	let mut app = APP_TEST_STATE.get().unwrap().write().await;

	//TODO add here the customer jwt when customer mod is done

	let url = get_url("api/v1/customer/app/".to_owned() + app.app_id.as_str() + "/token_renew");

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
	assert_ne!(out.secret_token, app.app_secret_token);
	assert_ne!(out.public_token, app.app_public_token);

	//set the new tokens
	app.app_secret_token = out.secret_token;
	app.app_public_token = out.public_token;
}

#[tokio::test]
async fn test_4_add_new_jwt_keys()
{
	//TODO add here the customer jwt when customer mod is done
	let mut app = APP_TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/customer/app/".to_owned() + app.app_id.as_str() + "/new_jwt_keys");

	let client = reqwest::Client::new();
	let res = client.patch(url).send().await.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<AppJwtRegisterOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	assert_ne!(
		out.jwt_sign_key,
		app.jwt_data.as_ref().unwrap()[0].jwt_sign_key.to_string()
	);
	assert_ne!(
		out.jwt_verify_key,
		app.jwt_data.as_ref().unwrap()[0].jwt_verify_key.to_string()
	);

	app.jwt_data.as_mut().unwrap().push(out);

	//TODO test login with old jwt keys (must be), in user tests
}

#[tokio::test]
async fn test_5_delete_jwt_keys()
{
	//TODO add here the customer jwt when customer mod is done
	let mut app = APP_TEST_STATE.get().unwrap().write().await;

	let jwt_id = &app.jwt_data.as_ref().unwrap()[0].jwt_id;

	let url = get_url("api/v1/customer/app/".to_owned() + app.app_id.as_str() + "/jwt/" + jwt_id);

	let client = reqwest::Client::new();
	let res = client.delete(url).send().await.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<JwtKeyDeleteOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	assert_eq!(out.old_jwt_id, jwt_id.to_string());

	app.jwt_data.as_mut().unwrap().remove(0);
}

#[tokio::test]
async fn test_6_delete_app()
{
	//TODO add here the customer jwt when customer mod is done

	let app = APP_TEST_STATE.get().unwrap().read().await;

	let app_id = &app.app_id;

	let url = get_url("api/v1/customer/app/".to_owned() + app_id);
	let client = reqwest::Client::new();
	let res = client.delete(url).send().await.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<AppDeleteOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();
	assert_eq!(out.old_app_id, app_id.to_string());
	assert_eq!(out.msg, "App deleted");
}

#[tokio::test]
async fn test_7_create_app_test_fn()
{
	let app_data = create_app().await;

	delete_app(app_data.app_id.as_str()).await;
}

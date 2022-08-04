use hyper::header::AUTHORIZATION;
use reqwest::StatusCode;
use sentc_crypto_common::ServerOutput;
use server_api_common::app::{AppJwtData, AppJwtRegisterOutput, AppRegisterInput, AppRegisterOutput, AppTokenRenewOutput};
use server_api_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{add_app_jwt_keys, auth_header, create_app, create_test_customer, customer_delete, delete_app, delete_app_jwt_key, get_url};

mod test_fn;

pub struct AppState
{
	pub app_id: String,
	pub app_public_token: String,
	pub app_secret_token: String,
	pub jwt_data: Option<Vec<AppJwtRegisterOutput>>,
	pub customer_data: CustomerDoneLoginOutput,
}

static APP_TEST_STATE: OnceCell<RwLock<AppState>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_global_test()
{
	APP_TEST_STATE
		.get_or_init(|| {
			async {
				let (_, customer_data) = create_test_customer("hello@test2.com", "12345").await;

				RwLock::new(AppState {
					app_id: "".to_string(),
					app_public_token: "".to_string(),
					app_secret_token: "".to_string(),
					jwt_data: None,
					customer_data,
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_1_create_app()
{
	let mut app = APP_TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/customer/app".to_owned());

	let input = AppRegisterInput {
		identifier: Some("My app".to_string()),
	};

	let customer_jwt = &app.customer_data.user_keys.jwt;

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
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

	app.app_public_token = out.public_token;
	app.app_secret_token = out.secret_token;
	app.app_id = out.app_id;
	app.jwt_data = Some(vec![out.jwt_data]);
}

#[tokio::test]
async fn test_2_update_app()
{
	let app = APP_TEST_STATE.get().unwrap().read().await;

	let customer_jwt = &app.customer_data.user_keys.jwt;

	let input = AppRegisterInput {
		identifier: Some("My app updated".to_string()),
	};

	let url = get_url("api/v1/customer/app/".to_owned() + app.app_id.as_str());

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	sentc_crypto::util_pub::handle_general_server_response(body.as_str()).unwrap();
}

#[tokio::test]
async fn test_3_renew_tokens()
{
	let mut app = APP_TEST_STATE.get().unwrap().write().await;

	let customer_jwt = &app.customer_data.user_keys.jwt;

	let url = get_url("api/v1/customer/app/".to_owned() + app.app_id.as_str() + "/token_renew");

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

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
	let mut app = APP_TEST_STATE.get().unwrap().write().await;

	let customer_jwt = &app.customer_data.user_keys.jwt;

	let url = get_url("api/v1/customer/app/".to_owned() + app.app_id.as_str() + "/new_jwt_keys");

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

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
}

#[tokio::test]
async fn test_5_get_app_jwt_keys()
{
	let app = APP_TEST_STATE.get().unwrap().read().await;

	let customer_jwt = &app.customer_data.user_keys.jwt;

	let url = get_url("api/v1/customer/app/".to_owned() + app.app_id.as_str() + "/jwt");

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	let out = ServerOutput::<Vec<AppJwtData>>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, true);
	assert_eq!(out.err_code, None);

	let out = out.result.unwrap();

	assert_eq!(out.len(), 2);
	assert_eq!(out[1].jwt_key_id, app.jwt_data.as_ref().unwrap()[0].jwt_id); //oder by time DESC
	assert_eq!(out[0].jwt_key_id, app.jwt_data.as_ref().unwrap()[1].jwt_id); //oder by time DESC
}

#[tokio::test]
async fn test_6_delete_jwt_keys()
{
	let mut app = APP_TEST_STATE.get().unwrap().write().await;

	let customer_jwt = &app.customer_data.user_keys.jwt;

	let jwt_id = &app.jwt_data.as_ref().unwrap()[0].jwt_id;

	let url = get_url("api/v1/customer/app/".to_owned() + app.app_id.as_str() + "/jwt/" + jwt_id);

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	sentc_crypto::util_pub::handle_general_server_response(body.as_str()).unwrap();

	app.jwt_data.as_mut().unwrap().remove(0);
}

#[tokio::test]
async fn test_7_delete_app()
{
	let app = APP_TEST_STATE.get().unwrap().read().await;

	let customer_jwt = &app.customer_data.user_keys.jwt;

	let app_id = &app.app_id;

	let url = get_url("api/v1/customer/app/".to_owned() + app_id);
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();

	sentc_crypto::util_pub::handle_general_server_response(body.as_str()).unwrap();
}

#[tokio::test]
async fn test_8_create_app_test_fn()
{
	let app = APP_TEST_STATE.get().unwrap().read().await;

	let customer_jwt = &app.customer_data.user_keys.jwt;

	let app_data = create_app(customer_jwt).await;

	add_app_jwt_keys(customer_jwt, app_data.app_id.as_str()).await;

	delete_app_jwt_key(
		customer_jwt,
		app_data.app_id.as_str(),
		app_data.jwt_data.jwt_id.as_str(),
	)
	.await;

	delete_app(customer_jwt, app_data.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

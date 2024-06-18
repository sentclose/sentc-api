//mfa tests

use reqwest::header::AUTHORIZATION;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::{StdUser, StdUserDataInt};
use sentc_crypto_common::user::{OtpRecoveryKeysOutput, OtpRegister, UserForcedAction};
use sentc_crypto_common::UserId;
use server_dashboard_common::app::AppRegisterOutput;
use server_dashboard_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};
use totp_rs::{Algorithm, Secret, TOTP};

use crate::test_fn::{
	auth_header,
	create_app,
	create_test_customer,
	create_test_user,
	customer_delete,
	delete_app,
	delete_user,
	get_base_url,
	get_url,
	login_user,
};

mod test_fn;

pub struct UserState
{
	pub username: String,
	pub pw: String,
	pub user_id: UserId,
	pub user_data: StdUserDataInt,
	pub app_data: AppRegisterOutput,
	pub customer_data: CustomerDoneLoginOutput,
	pub otp_secret: String,
	pub recover: Vec<String>,
}

static USER_TEST_STATE: OnceCell<RwLock<UserState>> = OnceCell::const_new();

fn get_totp(sec: String) -> TOTP
{
	TOTP::new(Algorithm::SHA256, 6, 1, 30, Secret::Encoded(sec).to_bytes().unwrap()).unwrap()
}

#[tokio::test]
async fn aaa_init_global_test()
{
	dotenv::from_filename("sentc.env").ok();

	let (_, customer_data) = create_test_customer("hello@test3.com", "12345").await;

	let customer_jwt = &customer_data.verify.jwt;

	//create here an app
	let app_data = create_app(customer_jwt).await;

	let (_user_id, user_data) = create_test_user(&app_data.secret_token, &app_data.public_token, "admin_test", "12345").await;

	USER_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(UserState {
					username: "admin_test".to_string(),
					pw: "12345".to_string(),
					user_id: "".to_string(),
					user_data,
					app_data,
					customer_data,
					otp_secret: "".to_string(),
					recover: Default::default(),
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_10_enable_otp()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/user/register_otp".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&user.user_data.jwt))
		.header("x-sentc-app-token", &user.app_data.public_token)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	let out: OtpRegister = handle_server_response(&body).unwrap();

	user.otp_secret = out.secret;
	user.recover = out.recover;
}

#[tokio::test]
async fn test_11_should_login_with_otp()
{
	//make the pre req to the sever with the username
	let url = get_url("api/v1/prepare_login".to_owned());

	let user = USER_TEST_STATE.get().unwrap().read().await;
	let username = &user.username;
	let pw = &user.pw;

	let prep_server_input = sentc_crypto::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = StdUser::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let server_out = res.text().await.unwrap();

	let r = sentc_crypto::user::check_done_login(&server_out).unwrap();

	match r {
		sentc_crypto::sdk_common::user::DoneLoginServerReturn::Otp => {},
		_ => {
			panic!("Should be otp login")
		},
	}

	let url = get_url("api/v1/validate_mfa".to_owned());

	//create a token
	let totp = get_totp(user.otp_secret.clone());

	let token = totp.generate_current().unwrap();

	let input = sentc_crypto::user::prepare_validate_mfa(auth_key.clone(), username.clone(), token).unwrap();

	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let server_out = res.text().await.unwrap();

	let keys = StdUser::done_validate_mfa(&derived_master_key, auth_key, username.clone(), &server_out).unwrap();

	let url = get_url("api/v1/verify_login".to_owned());
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(keys.challenge)
		.send()
		.await
		.unwrap();
	let server_out = res.text().await.unwrap();

	let keys = StdUser::verify_login(&server_out, keys.user_id, keys.device_id, keys.device_keys).unwrap();

	assert_eq!(
		user.user_data.device_keys.private_key.key_id,
		keys.device_keys.private_key.key_id
	);
}

#[tokio::test]
async fn test_12_get_all_recovery_keys()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/user/otp_recovery_keys".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&user.user_data.jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: OtpRecoveryKeysOutput = handle_server_response(&body).unwrap();

	//no key should be taken
	assert_eq!(out.keys.len(), user.recover.len());
}

#[tokio::test]
async fn test_13_login_with_recovery_key()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;
	let token = &user.recover[0];

	let url = get_url("api/v1/prepare_login".to_owned());

	let username = &user.username;
	let pw = &user.pw;

	let prep_server_input = sentc_crypto::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = StdUser::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let server_out = res.text().await.unwrap();

	let _r = sentc_crypto::user::check_done_login(&server_out).unwrap();

	let keys = StdUser::mfa_login(
		get_base_url(),
		&user.app_data.public_token,
		&derived_master_key,
		auth_key,
		username.clone(),
		token.clone(),
		true,
	)
	.await
	.unwrap();

	assert_eq!(
		user.user_data.device_keys.private_key.key_id,
		keys.device_keys.private_key.key_id
	);
}

#[tokio::test]
async fn test_14_get_all_recovery_keys_minus_one_after_usage()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/user/otp_recovery_keys".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&user.user_data.jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: OtpRecoveryKeysOutput = handle_server_response(&body).unwrap();

	//no key should be taken
	assert_eq!(out.keys.len(), user.recover.len() - 1);
}

#[tokio::test]
async fn test_15_not_use_key_again()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;
	let token = &user.recover[0];

	let url = get_url("api/v1/prepare_login".to_owned());

	let username = &user.username;
	let pw = &user.pw;

	let prep_server_input = sentc_crypto::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = StdUser::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let server_out = res.text().await.unwrap();

	let _r = sentc_crypto::user::check_done_login(&server_out).unwrap();

	let keys = StdUser::mfa_login(
		get_base_url(),
		&user.app_data.public_token,
		&derived_master_key,
		auth_key,
		username.clone(),
		token.clone(),
		true,
	)
	.await;

	match keys {
		Ok(_) => panic!("should be an error for used recovery key"),
		Err(_e) => {},
	}
}

#[tokio::test]
async fn test_16_test_with_all_keys()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;
	let url = get_url("api/v1/prepare_login".to_owned());

	let username = &user.username;
	let pw = &user.pw;

	let prep_server_input = sentc_crypto::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = StdUser::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let server_out = res.text().await.unwrap();

	let _r = sentc_crypto::user::check_done_login(&server_out).unwrap();

	//now itr over each key

	for (i, token) in user.recover.iter().enumerate() {
		if i == 0 {
			//this key was used in the test before
			continue;
		}

		let keys = StdUser::mfa_login(
			get_base_url(),
			&user.app_data.public_token,
			&derived_master_key,
			auth_key.clone(),
			username.clone(),
			token.clone(),
			true,
		)
		.await
		.unwrap();

		assert_eq!(
			user.user_data.device_keys.private_key.key_id,
			keys.device_keys.private_key.key_id
		);
	}
}

#[tokio::test]
async fn test_17_get_not_key_after_used_all_of_them()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/user/otp_recovery_keys".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&user.user_data.jwt))
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: OtpRecoveryKeysOutput = handle_server_response(&body).unwrap();

	assert_eq!(out.keys.len(), 0);
}

#[tokio::test]
async fn test_18_reset_otp()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/user/reset_otp".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&user.user_data.jwt))
		.header("x-sentc-app-token", &user.app_data.public_token)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	let out: OtpRegister = handle_server_response(&body).unwrap();

	user.otp_secret = out.secret;
	user.recover = out.recover;
}

#[tokio::test]
async fn test_19_login_with_new_totp_secret()
{
	//make the pre req to the sever with the username
	let url = get_url("api/v1/prepare_login".to_owned());

	let user = USER_TEST_STATE.get().unwrap().read().await;
	let username = &user.username;
	let pw = &user.pw;

	let prep_server_input = sentc_crypto::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = StdUser::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let server_out = res.text().await.unwrap();

	let r = sentc_crypto::user::check_done_login(&server_out).unwrap();

	match r {
		sentc_crypto::sdk_common::user::DoneLoginServerReturn::Otp => {},
		_ => {
			panic!("Should be otp login")
		},
	}

	//create a token
	let totp = get_totp(user.otp_secret.clone());

	let token = totp.generate_current().unwrap();

	let keys = StdUser::mfa_login(
		get_base_url(),
		&user.app_data.public_token,
		&derived_master_key,
		auth_key,
		username.clone(),
		token.clone(),
		false,
	)
	.await
	.unwrap();

	assert_eq!(
		user.user_data.device_keys.private_key.key_id,
		keys.device_keys.private_key.key_id
	);
}

#[tokio::test]
async fn test_20_login_with_new_recovery_keys()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;
	let token = &user.recover[0];

	let url = get_url("api/v1/prepare_login".to_owned());

	let username = &user.username;
	let pw = &user.pw;

	let prep_server_input = sentc_crypto::user::prepare_login_start(username.as_str()).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(prep_server_input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let (input, auth_key, derived_master_key) = StdUser::prepare_login(username, pw, body.as_str()).unwrap();

	// //done login
	let url = get_url("api/v1/done_login".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.public_token)
		.body(input)
		.send()
		.await
		.unwrap();

	let server_out = res.text().await.unwrap();

	let _r = sentc_crypto::user::check_done_login(&server_out).unwrap();

	let keys = StdUser::mfa_login(
		get_base_url(),
		&user.app_data.public_token,
		&derived_master_key,
		auth_key,
		username.clone(),
		token.clone(),
		true,
	)
	.await
	.unwrap();

	assert_eq!(
		user.user_data.device_keys.private_key.key_id,
		keys.device_keys.private_key.key_id
	);
}

#[tokio::test]
async fn test_21_reset_otp_with_recover_keys_left()
{
	let mut user = USER_TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/user/reset_otp".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&user.user_data.jwt))
		.header("x-sentc-app-token", &user.app_data.public_token)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	let out: OtpRegister = handle_server_response(&body).unwrap();

	user.otp_secret = out.secret;
	user.recover = out.recover;
}

#[tokio::test]
async fn test_22_forced_login()
{
	//should work even if otp is enabled

	let user = USER_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/user/forced/login".to_owned());

	let input = UserForcedAction {
		user_identifier: user.username.clone(),
	};

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	let _out: sentc_crypto_common::user::LoginForcedOutput = handle_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_23_forced_login_light()
{
	//should work even if otp is enabled

	let user = USER_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/user/forced/login_light".to_owned());

	let input = UserForcedAction {
		user_identifier: user.username.clone(),
	};

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	let _out: sentc_crypto_common::user::LoginForcedLightOutput = handle_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_24_disable_otp()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;
	let url = get_url("api/v1/user/disable_otp".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&user.user_data.jwt))
		.header("x-sentc-app-token", &user.app_data.public_token)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_25_login_normal_after_disable_otp()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;

	let keys = login_user(&user.app_data.public_token, &user.username, &user.pw).await;

	assert_eq!(
		user.user_data.device_keys.private_key.key_id,
		keys.device_keys.private_key.key_id
	);
}

#[tokio::test]
async fn test_26_disable_otp_forced()
{
	//first enable otp
	let user = USER_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/user/register_otp".to_owned());

	let client = reqwest::Client::new();
	let res = client
		.patch(url)
		.header(AUTHORIZATION, auth_header(&user.user_data.jwt))
		.header("x-sentc-app-token", &user.app_data.public_token)
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	let _out: OtpRegister = handle_server_response(&body).unwrap();

	//now disable it via forced
	let url = get_url("api/v1/user/forced/disable_otp".to_owned());

	let input = UserForcedAction {
		user_identifier: user.username.clone(),
	};

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", &user.app_data.secret_token)
		.body(serde_json::to_string(&input).unwrap())
		.send()
		.await
		.unwrap();
	let body = res.text().await.unwrap();

	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_26_login_normal_after_disable_otp_forced()
{
	let user = USER_TEST_STATE.get().unwrap().read().await;

	let keys = login_user(&user.app_data.public_token, &user.username, &user.pw).await;

	assert_eq!(
		user.user_data.device_keys.private_key.key_id,
		keys.device_keys.private_key.key_id
	);
}

#[tokio::test]
async fn zzz_clean_up()
{
	let user = &USER_TEST_STATE.get().unwrap().read().await;
	let customer_jwt = &user.customer_data.verify.jwt;

	delete_user(&user.app_data.secret_token, user.username.clone()).await;

	delete_app(customer_jwt, user.app_data.app_id.as_str()).await;

	customer_delete(customer_jwt).await;
}

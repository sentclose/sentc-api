use std::future::Future;
use std::ptr;

use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto::sdk_core::HashedAuthenticationKey;
use sentc_crypto_common::user::{DoneLoginServerInput, OtpInput, PrepareLoginSaltServerOutput, VerifyLoginInput};
use sentc_crypto_common::AppId;
use server_api_common::customer_app::app_entities::AppData;
use server_api_common::user::jwt::create_jwt;
use server_api_common::util::hash_token_to_string;

use crate::sentc_user_entities::{DoneLoginServerOutput, DoneLoginServerReturn, VerifyLoginEntity, VerifyLoginForcedEntity, SERVER_RANDOM_VALUE};
use crate::sentc_user_service::create_refresh_token;
use crate::user::auth::auth_model;
use crate::user::otp;
use crate::util::api_res::ApiErrorCodes;

pub fn prepare_login<'a>(app_data: &'a AppData, user_identifier: &'a str) -> impl Future<Output = AppRes<PrepareLoginSaltServerOutput>> + 'a
{
	create_salt(&app_data.app_data.app_id, user_identifier)
}

/**
Prepare the challenge and give the device keys back.

Use this fn directly after done login if the user does not enable 2fa-
But if so then use it after validate otp
 */
pub(crate) async fn prepare_done_login(app_id: impl Into<AppId>, identifier: impl Into<String>) -> AppRes<DoneLoginServerOutput>
{
	let app_id = app_id.into();

	//if correct -> fetch and return the user data
	let device_keys = auth_model::get_done_login_data(&app_id, identifier)
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(401, ApiErrorCodes::Login, "Wrong username or password"))?;

	let challenge = create_refresh_token()?;

	let encrypted_challenge = sentc_crypto::util::server::encrypt_login_verify_challenge(
		&device_keys.public_key_string,
		&device_keys.keypair_encrypt_alg,
		&challenge,
	)
	.map_err(|_e| {
		ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppTokenWrongFormat,
			"Can't create login challenge",
		)
	})?;

	auth_model::insert_verify_login_challenge(app_id, &device_keys.device_id, challenge).await?;

	let out = DoneLoginServerOutput {
		device_keys,
		challenge: encrypted_challenge,
	};

	Ok(out)
}

/**
After successful login return the user keys, so they can be decrypted in the client
 */
pub async fn done_login(app_data: &AppData, done_login: DoneLoginServerInput) -> AppRes<DoneLoginServerReturn>
{
	let identifier = hash_token_to_string(done_login.device_identifier.as_bytes())?;

	let (_, sec) = auth_user_mfa(&app_data.app_data.app_id, &identifier, done_login.auth_key).await?;

	if sec.is_none() {
		Ok(DoneLoginServerReturn::Direct(
			prepare_done_login(&app_data.app_data.app_id, identifier).await?,
		))
	} else {
		Ok(DoneLoginServerReturn::Otp)
	}
}

pub async fn validate_mfa(app_data: &AppData, input: OtpInput) -> AppRes<DoneLoginServerOutput>
{
	let identifier = hash_token_to_string(input.device_identifier.as_bytes())?;
	let (_, sec) = auth_user_mfa(&app_data.app_data.app_id, &identifier, input.auth_key).await?;

	//an error here because if user calls this fn it must be otp enabled
	let sec = sec.ok_or_else(|| ServerCoreError::new_msg(400, ApiErrorCodes::ToTpGet, "Otp secret not found"))?;

	//encrypt it here and not in the model because the real secret is only needed here and not for the checks if user enabled 2fa
	let sec = encrypted_at_rest_root::decrypt(&sec).await?;

	if !otp::validate_otp(sec, &input.token)? {
		return Err(ServerCoreError::new_msg(
			402,
			ApiErrorCodes::ToTpWrongToken,
			"Wrong otp.",
		));
	}

	//if we add more factors for the auth in the future then validate them in this fn, get it from auth_user_otp

	prepare_done_login(&app_data.app_data.app_id, identifier).await
}

pub async fn validate_recovery_otp(app_data: &AppData, input: OtpInput) -> AppRes<DoneLoginServerOutput>
{
	//the token is the recovery token. the secrete of the totp can be ignored because we are using the recovery tokens.
	let identifier = hash_token_to_string(input.device_identifier.as_bytes())?;
	auth_user(&app_data.app_data.app_id, &identifier, input.auth_key).await?;

	let hashed_token = hash_token_to_string(input.token.as_bytes())?;

	let token_id = auth_model::get_otp_recovery_token(&app_data.app_data.app_id, &identifier, hashed_token).await?;

	let done_login = prepare_done_login(&app_data.app_data.app_id, identifier).await?;

	//now delete the token but only after done login fetch makes no problems
	auth_model::delete_otp_recovery_token(token_id).await?;

	Ok(done_login)
}

pub(crate) async fn verify_login_internally(app_data: &AppData, done_login: VerifyLoginInput) -> AppRes<(VerifyLoginEntity, String, String)>
{
	let identifier = hash_token_to_string(done_login.device_identifier.as_bytes())?;
	auth_user(&app_data.app_data.app_id, &identifier, done_login.auth_key).await?;

	//verify the login, return the device and user id and user group id
	let data = auth_model::get_verify_login_data(&app_data.app_data.app_id, identifier, done_login.challenge)
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(401, ApiErrorCodes::Login, "Wrong username or password"))?;

	// and create the jwt
	let jwt = create_jwt(
		&data.user_id,
		&data.device_id,
		&app_data.jwt_data[0], //use always the latest created jwt data
		true,
	)
	.await?;

	let refresh_token = create_refresh_token()?;

	//activate refresh token
	auth_model::insert_refresh_token(&app_data.app_data.app_id, &data.device_id, &refresh_token).await?;

	Ok((data, jwt, refresh_token))
}

pub(crate) async fn verify_login_forced_internally(app_data: &AppData, identifier: &str) -> AppRes<(VerifyLoginForcedEntity, String, String)>
{
	let identifier = hash_token_to_string(identifier.as_bytes())?;

	//No auth user here because this is only called from a backend

	//verify the login, return the device and user id and user group id
	let data = auth_model::get_verify_login_data_forced(&app_data.app_data.app_id, identifier)
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(401, ApiErrorCodes::Login, "Wrong username or password"))?;

	// and create the jwt
	let jwt = create_jwt(
		&data.user_id,
		&data.device_id,
		&app_data.jwt_data[0], //use always the latest created jwt data
		true,
	)
	.await?;

	let refresh_token = create_refresh_token()?;

	//activate refresh token
	auth_model::insert_refresh_token(&app_data.app_data.app_id, &data.device_id, &refresh_token).await?;

	Ok((data, jwt, refresh_token))
}

async fn create_salt(app_id: impl Into<AppId>, user_identifier: &str) -> AppRes<PrepareLoginSaltServerOutput>
{
	let identifier = hash_token_to_string(user_identifier.as_bytes())?;

	//check the user id in the db
	let login_data = auth_model::get_user_login_data(app_id, &identifier).await?;

	//create the salt
	let (client_random_value, alg, add_str) = match login_data {
		Some(d) => (d.client_random_value, d.derived_alg, ""),

		//when user_identifier not found, push the user_identifier to the salt string, use the default alg
		None => {
			(
				SERVER_RANDOM_VALUE.0.to_owned(),
				SERVER_RANDOM_VALUE.1.to_owned(),
				user_identifier,
			)
		},
	};

	let salt_string = sentc_crypto::util::server::generate_salt_from_base64_to_string(client_random_value.as_str(), alg.as_str(), add_str)
		.map_err(|_e| ServerCoreError::new_msg(401, ApiErrorCodes::SaltError, "Can't create salt"))?;

	let out = PrepareLoginSaltServerOutput {
		salt_string,
		derived_encryption_key_alg: alg,
	};

	Ok(out)
}

//__________________________________________________________________________________________________

fn auth_user_private(auth_key: &str, hashed_user_auth_key: &str, alg: &str) -> AppRes<()>
{
	//hash the auth key and use the first 16 bytes
	let (server_hashed_auth_key, hashed_client_key) = sentc_crypto::util::server::get_auth_keys_from_base64(auth_key, hashed_user_auth_key, alg)
		.map_err(|_e| {
			ServerCoreError::new_msg(
				401,
				ApiErrorCodes::AuthKeyFormat,
				"The authentication key has a wrong format",
			)
		})?;

	//check the keys
	let check = compare_auth_keys(server_hashed_auth_key, hashed_client_key);

	//if not correct -> err msg
	if !check {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::Login,
			"Wrong username or password",
		));
	}

	Ok(())
}

pub(crate) async fn auth_user(app_id: &str, hashed_user_identifier: impl Into<String>, auth_key: String) -> AppRes<String>
{
	//split get user login into simple without table join and extend for the otp

	//get the login data
	let login_data = auth_model::get_user_login_data(app_id, hashed_user_identifier).await?;

	let (hashed_user_auth_key, alg) = match login_data {
		Some(d) => (d.hashed_authentication_key, d.derived_alg),
		None => {
			return Err(ServerCoreError::new_msg(
				401,
				ApiErrorCodes::UserNotFound,
				"No user found with this identifier",
			))
		},
	};

	auth_user_private(&auth_key, &hashed_user_auth_key, &alg)?;

	//return this here for the update user pw functions
	Ok(hashed_user_auth_key)
}

pub(super) async fn auth_user_mfa(app_id: &str, hashed_user_identifier: impl Into<String>, auth_key: String) -> AppRes<(String, Option<String>)>
{
	//get the login data
	let login_data = auth_model::get_user_login_data_with_otp(app_id, hashed_user_identifier).await?;

	let (hashed_user_auth_key, alg, otp_secret) = match login_data {
		Some(d) => (d.hashed_authentication_key, d.derived_alg, d.otp_secret),
		None => {
			return Err(ServerCoreError::new_msg(
				401,
				ApiErrorCodes::UserNotFound,
				"No user found with this identifier",
			))
		},
	};

	auth_user_private(&auth_key, &hashed_user_auth_key, &alg)?;

	//return this here for the update user pw functions
	Ok((hashed_user_auth_key, otp_secret))
}

/// Secure `memeq`.
/// from here:https://github.com/quininer/memsec/blob/master/src/lib.rs#L22
#[inline(never)]
unsafe fn memeq(b1: *const u8, b2: *const u8, len: usize) -> bool
{
	(0..len)
		.map(|i| ptr::read_volatile(b1.add(i)) ^ ptr::read_volatile(b2.add(i)))
		.fold(0, |sum, next| sum | next)
		.eq(&0)
}

fn compare_auth_keys(left: HashedAuthenticationKey, right: HashedAuthenticationKey) -> bool
{
	match (left, right) {
		(HashedAuthenticationKey::Argon2(l), HashedAuthenticationKey::Argon2(r)) => {
			//calling in unsafe block

			unsafe {
				//
				memeq(l.as_ptr(), r.as_ptr(), 16)
			}
		},
	}
}

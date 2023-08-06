use std::future::Future;

use rustgram_server_util::db::{exec, query_first, StringEntity};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::{get_time, set_params};
use sentc_crypto_common::{AppId, DeviceId};

use crate::sentc_user_entities::{DoneLoginServerKeysOutputEntity, UserLoginDataEntity, UserLoginDataOtpEntity, VerifyLoginEntity};
use crate::util::api_res::ApiErrorCodes;

/**
Internal login data

used for salt creation and auth user.

Get it for a device
 */
pub(super) fn get_user_login_data(
	app_id: impl Into<AppId>,
	user_identifier: impl Into<String>,
) -> impl Future<Output = AppRes<Option<UserLoginDataEntity>>>
{
	//language=SQL
	let sql = r"
SELECT client_random_value, hashed_auth_key, derived_alg 
FROM 
    sentc_user_device 
WHERE 
    device_identifier = ? AND 
    app_id = ? AND 
    user_id != 'not_registered'";

	query_first(sql, set_params!(user_identifier.into(), app_id.into()))
}

/**
Internal login data

used for salt creation and auth user.

Get it for a device
 */
pub(super) fn get_user_login_data_with_otp(
	app_id: impl Into<AppId>,
	user_identifier: impl Into<String>,
) -> impl Future<Output = AppRes<Option<UserLoginDataOtpEntity>>>
{
	//language=SQL
	let sql = r"
SELECT client_random_value, hashed_auth_key, derived_alg, otp_secret, otp_alg 
FROM 
    sentc_user_device ud, sentc_user u
WHERE 
    device_identifier = ? AND 
    ud.app_id = ? AND 
    user_id = u.id";

	query_first(sql, set_params!(user_identifier.into(), app_id.into()))
}

/**
The user data which are needed to get the user keys
 */
pub(super) async fn get_done_login_data(
	app_id: impl Into<AppId>,
	user_identifier: impl Into<String>,
) -> AppRes<Option<DoneLoginServerKeysOutputEntity>>
{
	//language=SQL
	let sql = r"
SELECT 
    encrypted_master_key,
    encrypted_private_key,
    public_key,
    keypair_encrypt_alg,
    encrypted_sign_key,
    verify_key,
    keypair_sign_alg,
    ud.id as k_id,
    user_id,
    user_group_id
FROM 
    sentc_user u, 
    sentc_user_device ud
WHERE 
    user_id = u.id AND
    ud.device_identifier = ? AND 
    u.app_id = ?";

	let data: Option<DoneLoginServerKeysOutputEntity> = query_first(sql, set_params!(user_identifier.into(), app_id.into())).await?;

	Ok(data)
}

pub(super) async fn insert_refresh_token(app_id: impl Into<AppId>, device_id: impl Into<DeviceId>, refresh_token: impl Into<String>) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_user_token (device_id, token, app_id, time) VALUES (?,?,?,?)";

	exec(
		sql,
		set_params!(
			device_id.into(),
			refresh_token.into(),
			app_id.into(),
			time.to_string()
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn insert_verify_login_challenge(app_id: impl Into<AppId>, device_id: impl Into<DeviceId>, challenge: String) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = r"
INSERT INTO sentc_user_device_challenge 
    (challenge, device_id, app_id, time) 
VALUES (?,?,?,?)";

	exec(
		sql,
		set_params!(challenge, device_id.into(), app_id.into(), time.to_string()),
	)
	.await
}

pub(super) async fn get_verify_login_data(
	app_id: impl Into<AppId>,
	user_identifier: impl Into<String>,
	challenge: String,
) -> AppRes<Option<VerifyLoginEntity>>
{
	//language=SQL
	let sql = r"
SELECT 
    ud.id as k_id,
    user_id,
    user_group_id
FROM 
    sentc_user u, 
    sentc_user_device ud, 
    sentc_user_device_challenge udc
WHERE 
    user_id = u.id AND 
    device_id = ud.id AND
    ud.device_identifier = ? AND 
    u.app_id = ? AND 
    challenge = ?";

	let out: Option<VerifyLoginEntity> = query_first(sql, set_params!(user_identifier.into(), app_id.into(), challenge)).await?;

	if let Some(o) = &out {
		//if challenge was found, delete it.
		//language=SQL
		let sql = "DELETE FROM sentc_user_device_challenge WHERE device_id = ?";

		exec(sql, set_params!(o.device_id.clone())).await?;
	}

	Ok(out)
}

pub(super) async fn get_verify_login_data_forced(app_id: impl Into<AppId>, user_identifier: impl Into<String>) -> AppRes<Option<VerifyLoginEntity>>
{
	//the same as verify login but this time without the challenge check

	//language=SQL
	let sql = r"
SELECT 
    ud.id as k_id,
    user_id,
    user_group_id
FROM 
    sentc_user u, 
    sentc_user_device ud
WHERE 
    user_id = u.id AND 
    ud.device_identifier = ? AND 
    u.app_id = ?";

	let out: Option<VerifyLoginEntity> = query_first(sql, set_params!(user_identifier.into(), app_id.into())).await?;

	Ok(out)
}

pub(super) async fn get_otp_recovery_token(app_id: impl Into<AppId>, user_identifier: impl Into<String>, hashed_token: String) -> AppRes<String>
{
	//check if the token exists.
	//in two fn to delete only the token after the login data was fetched.

	//language=SQL
	let sql = r"
SELECT r.id as recovery_id 
FROM 
    sentc_user_otp_recovery r, 
    sentc_user_device ud 
WHERE 
    app_id = ? AND 
    r.user_id = ud.user_id AND 
    device_identifier = ? AND 
    r.token_hash = ?";

	let out: StringEntity = query_first(sql, set_params!(app_id.into(), user_identifier.into(), hashed_token))
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(400, ApiErrorCodes::ToTpGet, "Recovery token not found"))?;

	Ok(out.0)
}

pub(super) fn delete_otp_recovery_token(token_id: String) -> impl Future<Output = AppRes<()>>
{
	//no other params here because this fn is never called directly with user input but after the token check

	//language=SQL
	let sql = "DELETE FROM sentc_user_otp_recovery WHERE id = ?";

	exec(sql, set_params!(token_id))
}

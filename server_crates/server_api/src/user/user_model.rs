use std::future::Future;

use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::db::{
	bulk_insert,
	exec,
	exec_transaction,
	query,
	query_first,
	query_string,
	I32Entity,
	I64Entity,
	Params,
	StringEntity,
	TransactionData,
	TupleEntity,
};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::{get_time, set_params};
use sentc_crypto_common::user::{ChangePasswordData, KeyDerivedData, MasterKey, ResetPasswordData};
use sentc_crypto_common::{AppId, DeviceId, EncryptionKeyPairId, GroupId, SignKeyPairId, UserId};

use crate::sentc_user_entities::{UserLoginDataOtpEntity, VerifyLoginEntity};
use crate::user::user_entities::{
	CaptchaEntity,
	DoneLoginServerKeysOutputEntity,
	UserDeviceList,
	UserLoginDataEntity,
	UserPublicKeyDataEntity,
	UserRefreshTokenCheck,
	UserVerifyKeyDataEntity,
};
use crate::user::user_service::UserAction;
use crate::util::api_res::ApiErrorCodes;
use crate::util::get_begin_of_month;

pub(super) async fn get_jwt_sign_key(kid: impl Into<String>) -> AppRes<Option<String>>
{
	//language=SQL
	let sql = "SELECT sign_key FROM sentc_app_jwt_keys WHERE id = ?";

	let sign_key: Option<StringEntity> = query_first(sql, set_params!(kid.into())).await?;

	//decrypt the sign key with ear root
	match sign_key {
		Some(sk) => Ok(Some(encrypted_at_rest_root::decrypt(&sk.0).await?)),
		None => Ok(None),
	}
}

pub(super) async fn get_jwt_verify_key(kid: impl Into<String>) -> AppRes<Option<String>>
{
	//language=SQL
	let sql = "SELECT verify_key FROM sentc_app_jwt_keys WHERE id = ?";

	let sign_key: Option<StringEntity> = query_first(sql, set_params!(kid.into())).await?;

	Ok(sign_key.map(|i| i.0))
}

//__________________________________________________________________________________________________
//user

pub(super) async fn check_user_in_app(app_id: impl Into<AppId>, user_id: impl Into<UserId>) -> AppRes<bool>
{
	//language=SQL
	let sql = "SELECT 1 FROM sentc_user WHERE id = ? AND app_id = ? LIMIT 1";

	let exists: Option<I64Entity> = query_first(sql, set_params!(user_id.into(), app_id.into())).await?;

	match exists {
		Some(_) => Ok(true),
		None => Ok(false),
	}
}

pub(super) async fn check_user_exists(app_id: impl Into<AppId>, user_identifier: impl Into<String>) -> AppRes<bool>
{
	//language=SQL
	let sql = r"
SELECT 1 
FROM 
    sentc_user_device ud
WHERE 
    device_identifier = ? AND 
    app_id = ?
LIMIT 1";

	let exists: Option<I64Entity> = query_first(sql, set_params!(user_identifier.into(), app_id.into())).await?;

	match exists {
		Some(_) => Ok(true),
		None => Ok(false),
	}
}

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
    app_id = ?";

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

pub(super) async fn check_refresh_token(
	app_id: impl Into<AppId>,
	device_id: impl Into<DeviceId>,
	refresh_token: String,
) -> AppRes<Option<UserRefreshTokenCheck>>
{
	//language=SQL
	let sql = r"
SELECT user_id, device_identifier 
FROM 
    sentc_user_token ut,
    sentc_user_device ud
WHERE ut.app_id = ? AND 
      ut.device_id = ? AND 
      ut.token = ? AND 
      ud.id = ut.device_id";

	let exists: Option<UserRefreshTokenCheck> = query_first(sql, set_params!(app_id.into(), device_id.into(), refresh_token)).await?;

	Ok(exists)
}

pub(super) async fn get_user_group_id(app_id: impl Into<AppId>, user_id: impl Into<UserId>) -> AppRes<Option<StringEntity>>
{
	//language=SQL
	let sql = "SELECT user_group_id FROM sentc_user WHERE app_id = ? AND id = ?";

	let id: Option<StringEntity> = query_first(sql, set_params!(app_id.into(), user_id.into())).await?;

	Ok(id)
}

//__________________________________________________________________________________________________

pub(super) async fn get_public_key_by_id(
	app_id: impl Into<AppId>,
	user_id: impl Into<UserId>,
	public_key_id: impl Into<EncryptionKeyPairId>,
) -> AppRes<UserPublicKeyDataEntity>
{
	//language=SQL
	let sql = r"
SELECT gk.id, public_key, private_key_pair_alg, public_key_sig, public_key_sig_key_id 
FROM 
    sentc_user u, 
    sentc_group_keys gk
WHERE 
    user_group_id = group_id AND 
    u.app_id = ? AND 
    u.id = ? AND 
    gk.id = ?";

	let data: Option<UserPublicKeyDataEntity> = query_first(sql, set_params!(app_id.into(), user_id.into(), public_key_id.into())).await?;

	match data {
		Some(d) => Ok(d),
		None => {
			Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::UserNotFound,
				"Public key from this user not found",
			))
		},
	}
}

/**
Get just the public key data for this user
*/
pub(super) async fn get_public_key_data(app_id: impl Into<AppId>, user_id: impl Into<UserId>) -> AppRes<UserPublicKeyDataEntity>
{
	//language=SQL
	let sql = r"
SELECT gk.id, public_key, private_key_pair_alg, public_key_sig, public_key_sig_key_id
FROM 
    sentc_user u, 
    sentc_group_keys gk
WHERE 
    user_group_id = group_id AND 
    u.app_id = ? AND 
    u.id = ? 
ORDER BY gk.time DESC 
LIMIT 1";

	let data: Option<UserPublicKeyDataEntity> = query_first(sql, set_params!(app_id.into(), user_id.into())).await?;

	match data {
		Some(d) => Ok(d),
		None => {
			Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::UserNotFound,
				"Public key from this user not found",
			))
		},
	}
}

pub(super) async fn get_verify_key_by_id(
	app_id: impl Into<AppId>,
	user_id: impl Into<UserId>,
	verify_key_id: impl Into<SignKeyPairId>,
) -> AppRes<UserVerifyKeyDataEntity>
{
	//language=SQL
	let sql = r"
SELECT gk.id,verify_key, keypair_sign_alg
FROM 
    sentc_user u, 
    sentc_group_keys gk
WHERE 
    user_group_id = group_id AND 
    u.app_id = ? AND 
    u.id = ? AND 
    gk.id = ?";

	let data: Option<UserVerifyKeyDataEntity> = query_first(sql, set_params!(app_id.into(), user_id.into(), verify_key_id.into())).await?;

	match data {
		Some(d) => Ok(d),
		None => {
			Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::UserNotFound,
				"Verify key from this user not found",
			))
		},
	}
}

//__________________________________________________________________________________________________

/**
# Update the group user id

For register we don't know the group id of the user group.
The service must create a group first and then we can update the group id
*/
pub(super) async fn register_update_user_group_id(app_id: impl Into<AppId>, user_id: impl Into<UserId>, group_id: GroupId) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_user SET user_group_id = ? WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(group_id, user_id.into(), app_id.into())).await?;

	Ok(())
}

pub(super) async fn register(
	app_id: impl Into<AppId>,
	device_identifier: String,
	master_key: MasterKey,
	derived: KeyDerivedData,
) -> AppRes<(UserId, DeviceId)>
{
	let app_id = app_id.into();

	//check first if the user identifier is available
	let check = check_user_exists(&app_id, &device_identifier).await?;

	if check {
		//check true == user exists
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::UserExists,
			"User already exists",
		));
	}

	//data for the user table
	//language=SQL
	let sql_user = "INSERT INTO sentc_user (id, app_id, user_group_id, time) VALUES (?,?,?,?)";
	let user_id = create_id();
	let time = get_time()?;

	//insert a fake group id for now, and update the user group id when user group was created
	let user_params = set_params!(user_id.clone(), app_id.clone(), "none".to_string(), time.to_string());

	let device_id = create_id();

	//data for the user key table
	let (sql_keys, key_params) = prepare_register_device(
		&device_id,
		&user_id,
		app_id,
		time,
		device_identifier,
		master_key,
		derived,
		None,
	);

	exec_transaction(vec![
		TransactionData {
			sql: sql_user,
			params: user_params,
		},
		TransactionData {
			sql: sql_keys,
			params: key_params,
		},
	])
	.await?;

	Ok((user_id, device_id))
}

pub(super) async fn register_device(
	app_id: impl Into<AppId>,
	device_identifier: String,
	master_key: MasterKey,
	derived: KeyDerivedData,
	token: impl Into<String>,
) -> AppRes<DeviceId>
{
	let device_id = create_id();
	let time = get_time()?;

	let (sql_keys, key_params) = prepare_register_device(
		&device_id,
		"not_registered",
		app_id,
		time,
		device_identifier,
		master_key,
		derived,
		Some(token.into()),
	);

	exec(sql_keys, key_params).await?;

	Ok(device_id)
}

pub(super) async fn get_done_register_device(app_id: impl Into<AppId>, token: String) -> AppRes<DeviceId>
{
	if token.as_str() == "NULL" || token.as_str() == "null" {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::UserNotFound,
			"Device was not found for this token",
		));
	}

	//language=SQL
	let sql = "SELECT id FROM sentc_user_device WHERE app_id = ? AND token = ?";

	let out: Option<StringEntity> = query_first(sql, set_params!(app_id.into(), token)).await?;

	let device_id: DeviceId = match out {
		Some(id) => id.0,
		None => {
			return Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::UserNotFound,
				"Device was not found for this token",
			))
		},
	};

	Ok(device_id)
}

pub(super) async fn done_register_device(app_id: impl Into<AppId>, user_id: impl Into<UserId>, device_id: impl Into<DeviceId>) -> AppRes<()>
{
	//update the user id, in two fn because there can be an err for inserting user keys in user group
	//language=SQL
	let sql = "UPDATE sentc_user_device SET user_id = ?, token = NULL WHERE id = ? AND app_id = ?";
	exec(sql, set_params!(user_id.into(), device_id.into(), app_id.into())).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn delete(user_id: impl Into<UserId>, app_id: impl Into<AppId>) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_user WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(user_id.into(), app_id.into())).await?;

	Ok(())
}

pub(super) async fn delete_device(user_id: impl Into<UserId>, app_id: impl Into<AppId>, device_id: impl Into<DeviceId>) -> AppRes<()>
{
	let app_id = app_id.into();
	let user_id = user_id.into();

	//first check if it is the only device
	//language=SQL
	let sql = "SELECT COUNT(id) FROM sentc_user_device WHERE app_id = ? AND user_id = ? LIMIT 2";

	let device_count: Option<I64Entity> = query_first(sql, set_params!(app_id.clone(), user_id.clone())).await?;

	match device_count {
		Some(c) => {
			if c.0 < 2 {
				return Err(ServerCoreError::new_msg(
					400,
					ApiErrorCodes::UserDeviceDelete,
					"Can't delete the last device. Use user delete instead.",
				));
			}
		},
		None => {
			return Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::UserDeviceNotFound,
				"No device found",
			));
		},
	}

	//language=SQL
	let sql = "DELETE FROM sentc_user_device WHERE user_id = ? AND id = ? AND app_id = ?";

	exec(sql, set_params!(user_id, device_id.into(), app_id)).await?;

	Ok(())
}

pub(super) async fn update(
	user_id: impl Into<UserId>,
	device_id: impl Into<DeviceId>,
	app_id: impl Into<AppId>,
	user_identifier: String,
) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_user_device SET device_identifier = ? WHERE id = ? AND user_id = ? AND app_id = ?";

	exec(
		sql,
		set_params!(user_identifier, device_id.into(), user_id.into(), app_id.into()),
	)
	.await?;

	Ok(())
}

pub(super) async fn change_password(
	user_id: impl Into<UserId>,
	device_id: impl Into<DeviceId>,
	data: ChangePasswordData,
	old_hashed_auth_key: String,
) -> AppRes<()>
{
	//update pw for a device
	//the master key is still the same, only new encrypt by the new password

	//language=SQL
	let sql = r"
UPDATE sentc_user_device 
SET 
    client_random_value = ?,
    hashed_auth_key = ?, 
    encrypted_master_key = ?, 
    encrypted_master_key_alg = ?, 
    derived_alg = ? 
WHERE 
    user_id = ? AND 
    id = ? AND 
    hashed_auth_key = ?";

	exec(
		sql,
		set_params!(
			data.new_client_random_value,
			data.new_hashed_authentication_key,
			data.new_encrypted_master_key,
			data.new_encrypted_master_key_alg,
			data.new_derived_alg,
			user_id.into(),
			device_id.into(),
			old_hashed_auth_key
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn reset_password(user_id: impl Into<UserId>, device_id: impl Into<DeviceId>, data: ResetPasswordData) -> AppRes<()>
{
	let device_id = device_id.into();

	//reset only the actual device
	//create a new master key from the new password. the key pairs are still the same

	//get the first row (the key id) which we are updating

	//language=SQL
	let sql = "SELECT app_id FROM sentc_user_device WHERE id = ? ORDER BY time DESC LIMIT 1";

	let row: Option<StringEntity> = query_first(sql, set_params!(device_id.clone())).await?;

	let row = match row {
		Some(r) => r,
		None => {
			return Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::UserNotFound,
				"No keys to update",
			))
		},
	};

	//language=SQL
	let sql = r"
UPDATE sentc_user_device 
SET client_random_value = ?,
    hashed_auth_key = ?,
    encrypted_master_key = ?,
    master_key_alg = ?,
    encrypted_master_key_alg = ?,
    derived_alg = ?, 
    encrypted_private_key = ?, 
    encrypted_sign_key = ? 
WHERE 
    id = ? AND 
    user_id = ? AND 
    app_id = ?";

	exec(
		sql,
		set_params!(
			data.client_random_value,
			data.hashed_authentication_key,
			data.master_key.encrypted_master_key,
			data.master_key.master_key_alg,
			data.master_key.encrypted_master_key_alg,
			data.derived_alg,
			data.encrypted_private_key,
			data.encrypted_sign_key,
			device_id,
			user_id.into(),
			row.0
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn get_devices(
	app_id: impl Into<AppId>,
	user_id: impl Into<UserId>,
	last_fetched_time: u128,
	last_fetched_id: impl Into<DeviceId>,
) -> AppRes<Vec<UserDeviceList>>
{
	//language=SQL
	let sql = r"
SELECT id, time, device_identifier 
FROM sentc_user_device 
WHERE 
    app_id = ? AND 
    user_id = ?"
		.to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND time >= ? AND (time > ? OR (time = ? AND id > ?)) ORDER BY time, id LIMIT 50";
		(
			sql,
			set_params!(
				app_id.into(),
				user_id.into(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY time, id LIMIT 50";

		(sql, set_params!(app_id.into(), user_id.into(),))
	};

	let list: Vec<UserDeviceList> = query_string(sql, params).await?;

	Ok(list)
}

pub(super) async fn get_device_identifier(
	app_id: impl Into<AppId>,
	user_id: impl Into<UserId>,
	device_id: impl Into<DeviceId>,
) -> AppRes<Option<StringEntity>>
{
	//language=SQL
	let sql = "SELECT device_identifier FROM sentc_user_device WHERE id = ? AND user_id = ? AND app_id = ?";

	let device: Option<StringEntity> = query_first(sql, set_params!(device_id.into(), user_id.into(), app_id.into())).await?;

	Ok(device)
}

//__________________________________________________________________________________________________
//otp

pub(super) async fn register_otp(app_id: impl Into<AppId>, user_id: impl Into<UserId>, secret: &str, alg: String, recover: &[String]) -> AppRes<()>
{
	//store the recovery keys and the secret but also encrypted

	let key = encrypted_at_rest_root::get_key_map().await;

	let time = get_time()?;
	let user_id = user_id.into();

	let encrypted_secret = encrypted_at_rest_root::encrypt_with_key(&key, secret)?;

	//language=SQL
	let sql = "UPDATE sentc_user SET otp_secret = ?, otp_alg = ? WHERE id = ? AND app_id = ?";

	exec(
		sql,
		set_params!(encrypted_secret, alg, user_id.clone(), app_id.into()),
	)
	.await?;

	let encrypted_recover = recover
		.iter()
		.map(|i| encrypted_at_rest_root::encrypt_with_key(&key, i))
		.collect::<Result<Vec<String>, _>>()?;

	//language=SQL
	//let sql = "INSERT INTO sentc_user_otp_recovery (id, user_id, token, time) VALUES (?,?,?,?)";

	bulk_insert(
		true,
		"sentc_user_otp_recovery",
		&["id", "user_id", "token", "time"],
		encrypted_recover,
		|i| {
			let id = create_id();

			set_params!(id, user_id.clone(), i, time.to_string())
		},
	)
	.await?;

	Ok(())
}

pub(super) async fn get_otp_recovery_token(app_id: impl Into<AppId>, user_identifier: impl Into<String>, encrypted_token: String) -> AppRes<String>
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
    r.token = ?";

	let out: StringEntity = query_first(
		sql,
		set_params!(app_id.into(), user_identifier.into(), encrypted_token),
	)
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

pub(super) fn delete_all_otp_token(user_id: impl Into<UserId>) -> impl Future<Output = AppRes<()>>
{
	//for token or 2fa reset, no app id check needed because this is done in the jwt mw
	//only with fresh jwt
	//language=SQL
	let sql = "DELETE FROM sentc_user_otp_recovery WHERE user_id = ?";

	exec(sql, set_params!(user_id.into()))
}

pub(super)fn disable_otp(user_id: impl Into<UserId>)-> impl Future<Output = AppRes<()>>
{
	//update the user table and set the values to null
	//language=SQL
	let sql = "UPDATE sentc_user SET otp_secret = NULL, otp_alg = NULL WHERE id = ?";
	
	exec(sql,set_params!(user_id.into()))
}

pub(super) fn get_otp_recovery_keys(user_id: impl Into<UserId>) -> impl Future<Output = AppRes<Vec<StringEntity>>>
{
	//only with fresh jwt
	//tokens are encrypted

	//language=SQL
	let sql = "SELECT token FROM sentc_user_otp_recovery WHERE user_id = ?";

	query(sql, set_params!(user_id.into()))
}

//__________________________________________________________________________________________________

pub(super) async fn save_user_action(app_id: impl Into<AppId>, user_id: impl Into<UserId>, action: UserAction, amount: i64) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_user_action_log (user_id, time, action_id, app_id, amount) VALUES (?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			user_id.into(),
			time.to_string(),
			action.get_int_code(),
			app_id.into(),
			amount
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn get_group_key_rotations_in_actual_month(app_id: impl Into<AppId>, group_id: impl Into<GroupId>) -> AppRes<i32>
{
	let begin = get_begin_of_month()?;

	//language=SQL
	let sql = "SELECT COUNT(user_id) FROM sentc_user_action_log WHERE app_id = ? AND user_id = ? AND action_id = ? AND time >= ?";

	let out: I32Entity = query_first(
		sql,
		set_params!(
			app_id.into(),
			group_id.into(),
			UserAction::KeyRotation.get_int_code(),
			begin
		),
	)
	.await?
	.unwrap_or(TupleEntity(0));

	Ok(out.0)
}

//__________________________________________________________________________________________________

pub(super) async fn save_captcha_solution(app_id: impl Into<AppId>, solution: String) -> AppRes<String>
{
	let time = get_time()?;
	let captcha_id = create_id();

	//language=SQL
	let sql = "INSERT INTO sentc_captcha (id, app_id, solution, time) VALUES (?,?,?,?)";

	exec(
		sql,
		set_params!(captcha_id.clone(), app_id.into(), solution, time.to_string()),
	)
	.await?;

	Ok(captcha_id)
}

pub(super) async fn get_captcha_solution(id: impl Into<String>, app_id: impl Into<AppId>) -> AppRes<Option<CaptchaEntity>>
{
	//language=SQL
	let sql = "SELECT solution, time FROM sentc_captcha WHERE id = ? AND app_id = ?";

	let out: Option<CaptchaEntity> = query_first(sql, set_params!(id.into(), app_id.into())).await?;

	Ok(out)
}

pub(super) async fn delete_captcha(app_id: impl Into<AppId>, id: String) -> AppRes<()>
{
	//owned id because we got the id from the input

	//language=SQL
	let sql = "DELETE FROM sentc_captcha WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(id, app_id.into())).await?;

	Ok(())
}

pub(super) fn prepare_register_device(
	device_id: impl Into<DeviceId>,
	user_id: impl Into<UserId>,
	app_id: impl Into<AppId>,
	time: u128,
	device_identifier: String,
	master_key_info: MasterKey,
	derived_data: KeyDerivedData,
	token: Option<String>,
) -> (&'static str, Params)
{
	//language=SQL
	let sql_keys = r"
INSERT INTO sentc_user_device 
    (id, 
     user_id, 
     app_id, 
     device_identifier,
     client_random_value, 
     public_key, 
     encrypted_private_key, 
     keypair_encrypt_alg, 
     encrypted_sign_key, 
     verify_key, 
     keypair_sign_alg, 
     derived_alg, 
     encrypted_master_key, 
     master_key_alg, 
     encrypted_master_key_alg, 
     hashed_auth_key, 
     time,
     token
     ) 
VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

	let key_params = set_params!(
		device_id.into(),
		user_id.into(),
		app_id.into(),
		device_identifier,
		derived_data.client_random_value,
		derived_data.public_key,
		derived_data.encrypted_private_key,
		derived_data.keypair_encrypt_alg,
		derived_data.encrypted_sign_key,
		derived_data.verify_key,
		derived_data.keypair_sign_alg,
		derived_data.derived_alg,
		master_key_info.encrypted_master_key,
		master_key_info.master_key_alg,
		master_key_info.encrypted_master_key_alg,
		derived_data.hashed_authentication_key,
		time.to_string(),
		token
	);

	(sql_keys, key_params)
}

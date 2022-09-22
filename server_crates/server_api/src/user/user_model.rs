use sentc_crypto_common::user::{ChangePasswordData, KeyDerivedData, MasterKey, ResetPasswordData, UserDeviceRegisterInput};
use sentc_crypto_common::{AppId, DeviceId, EncryptionKeyPairId, GroupId, SignKeyPairId, UserId};
use server_core::db::{exec, exec_transaction, query_first, query_string, Params, TransactionData};
use server_core::{get_time, set_params};
use uuid::Uuid;

use crate::user::user_entities::{
	DoneLoginServerKeysOutputEntity,
	JwtSignKey,
	JwtVerifyKey,
	UserDeviceList,
	UserExistsEntity,
	UserLoginDataEntity,
	UserLoginLightEntity,
	UserPublicData,
	UserPublicKeyDataEntity,
	UserRefreshTokenCheck,
	UserResetPwCheck,
	UserVerifyKeyDataEntity,
};
use crate::user::user_service::UserAction;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

pub(super) async fn get_jwt_sign_key(kid: &str) -> AppRes<String>
{
	//language=SQL
	let sql = "SELECT sign_key FROM sentc_app_jwt_keys WHERE id = ?";

	let sign_key: Option<JwtSignKey> = query_first(sql, set_params!(kid.to_string())).await?;

	match sign_key {
		Some(k) => Ok(k.0),
		None => {
			Err(HttpErr::new(
				200,
				ApiErrorCodes::JwtKeyNotFound,
				"No matched key to this key id".to_string(),
				None,
			))
		},
	}
}

pub(super) async fn get_jwt_verify_key(kid: &str) -> AppRes<String>
{
	//language=SQL
	let sql = "SELECT verify_key FROM sentc_app_jwt_keys WHERE id = ?";

	let sign_key: Option<JwtVerifyKey> = query_first(sql, set_params!(kid.to_string())).await?;

	match sign_key {
		Some(k) => Ok(k.0),
		None => {
			Err(HttpErr::new(
				200,
				ApiErrorCodes::JwtKeyNotFound,
				"No matched key to this key id".to_string(),
				None,
			))
		},
	}
}

//__________________________________________________________________________________________________
//user

pub(super) async fn check_user_in_app(app_id: AppId, user_id: UserId) -> AppRes<bool>
{
	//language=SQL
	let sql = "SELECT 1 FROM sentc_user WHERE id = ? AND app_id = ? LIMIT 1";

	let exists: Option<UserExistsEntity> = query_first(sql, set_params!(user_id, app_id)).await?;

	match exists {
		Some(_) => Ok(true),
		None => Ok(false),
	}
}

pub(super) async fn check_user_exists(app_id: &str, user_identifier: &str) -> AppRes<bool>
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

	let exists: Option<UserExistsEntity> = query_first(sql, set_params!(user_identifier.to_string(), app_id.to_string())).await?;

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
pub(super) async fn get_user_login_data(app_id: AppId, user_identifier: &str) -> AppRes<Option<UserLoginDataEntity>>
{
	//language=SQL
	let sql = r"
SELECT client_random_value, hashed_auth_key, derived_alg 
FROM 
    sentc_user_device 
WHERE 
    device_identifier = ? AND 
    app_id = ?";

	let login_data: Option<UserLoginDataEntity> = query_first(sql, set_params!(user_identifier.to_string(), app_id)).await?;

	Ok(login_data)
}

/**
The user data which are needed to get the user keys
*/
pub(super) async fn get_done_login_data(app_id: &str, user_identifier: &str) -> AppRes<Option<DoneLoginServerKeysOutputEntity>>
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

	let data: Option<DoneLoginServerKeysOutputEntity> = query_first(sql, set_params!(user_identifier.to_string(), app_id.to_string())).await?;

	Ok(data)
}

pub(super) async fn get_done_login_light_data(app_id: &str, user_identifier: &str) -> AppRes<Option<UserLoginLightEntity>>
{
	//language=SQL
	let sql = r"
SELECT user_id, ud.id as device_id, user_group_id
FROM 
    sentc_user_device ud, 
    sentc_user u 
WHERE 
    device_identifier = ? AND 
    user_id = u.id AND 
    u.app_id = ?";

	let data: Option<UserLoginLightEntity> = query_first(sql, set_params!(user_identifier.to_string(), app_id.to_string())).await?;

	Ok(data)
}

pub(super) async fn insert_refresh_token(app_id: AppId, device_id: DeviceId, refresh_token: String) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_user_token (device_id, token, app_id, time) VALUES (?,?,?,?)";

	exec(sql, set_params!(device_id, refresh_token, app_id, time.to_string())).await?;

	Ok(())
}

pub(super) async fn check_refresh_token(app_id: AppId, device_id: DeviceId, refresh_token: String) -> AppRes<Option<UserRefreshTokenCheck>>
{
	//language=SQL
	let sql = r"
SELECT user_id, device_identifier, user_group_id 
FROM 
    sentc_user_token ut,
    sentc_user_device ud,
    sentc_user u
WHERE ut.app_id = ? AND 
      ut.device_id = ? AND 
      ut.token = ? AND 
      ud.id = ut.device_id AND 
      u.id = user_id";

	let exists: Option<UserRefreshTokenCheck> = query_first(sql, set_params!(app_id, device_id, refresh_token)).await?;

	Ok(exists)
}

//__________________________________________________________________________________________________

/**
Get the public and verify key data from this user.

The public data are from the user group.
*/
pub(super) async fn get_public_data(app_id: AppId, user_id: UserId) -> AppRes<UserPublicData>
{
	//verify and sign keys are always set for user group

	//join on group and not group user because we only need the group keys here and not the user group keys

	//language=SQL
	let sql = r"
SELECT gk.id, public_key, private_key_pair_alg, verify_key, keypair_sign_alg
FROM 
    sentc_user u, 
    sentc_group_keys gk
WHERE 
    user_group_id = group_id AND 
    u.app_id = ? AND 
    u.id = ? 
ORDER BY gk.time DESC 
LIMIT 1";

	let data: Option<UserPublicData> = query_first(sql, set_params!(app_id, user_id)).await?;

	match data {
		Some(d) => Ok(d),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::UserNotFound,
				"Public data from this user not found".to_string(),
				None,
			))
		},
	}
}

pub(super) async fn get_public_key_by_id(app_id: AppId, user_id: UserId, public_key_id: EncryptionKeyPairId) -> AppRes<UserPublicKeyDataEntity>
{
	//language=SQL
	let sql = r"
SELECT gk.id, public_key, private_key_pair_alg 
FROM 
    sentc_user u, 
    sentc_group_keys gk
WHERE 
    user_group_id = group_id AND 
    u.app_id = ? AND 
    u.id = ? AND 
    gk.id = ?";

	let data: Option<UserPublicKeyDataEntity> = query_first(sql, set_params!(app_id, user_id, public_key_id)).await?;

	match data {
		Some(d) => Ok(d),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::UserNotFound,
				"Public key from this user not found".to_string(),
				None,
			))
		},
	}
}

/**
Get just the public key data for this user
*/
pub(super) async fn get_public_key_data(app_id: AppId, user_id: UserId) -> AppRes<UserPublicKeyDataEntity>
{
	//language=SQL
	let sql = r"
SELECT gk.id, public_key, private_key_pair_alg 
FROM 
    sentc_user u, 
    sentc_group_keys gk
WHERE 
    user_group_id = group_id AND 
    u.app_id = ? AND 
    u.id = ? 
ORDER BY gk.time DESC 
LIMIT 1";

	let data: Option<UserPublicKeyDataEntity> = query_first(sql, set_params!(app_id, user_id)).await?;

	match data {
		Some(d) => Ok(d),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::UserNotFound,
				"Public key from this user not found".to_string(),
				None,
			))
		},
	}
}

pub(super) async fn get_verify_key_by_id(app_id: AppId, user_id: UserId, verify_key_id: SignKeyPairId) -> AppRes<UserVerifyKeyDataEntity>
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

	let data: Option<UserVerifyKeyDataEntity> = query_first(sql, set_params!(app_id, user_id, verify_key_id)).await?;

	match data {
		Some(d) => Ok(d),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::UserNotFound,
				"Verify key from this user not found".to_string(),
				None,
			))
		},
	}
}

/**
Get just the verify key data for this user
 */
pub(super) async fn get_verify_key_data(app_id: AppId, user_id: UserId) -> AppRes<UserVerifyKeyDataEntity>
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
    u.id = ? 
ORDER BY gk.time DESC 
LIMIT 1";

	let data: Option<UserVerifyKeyDataEntity> = query_first(sql, set_params!(app_id, user_id)).await?;

	match data {
		Some(d) => Ok(d),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::UserNotFound,
				"Verify key from this user not found".to_string(),
				None,
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
pub(super) async fn register_update_user_group_id(app_id: AppId, user_id: UserId, group_id: GroupId) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_user SET user_group_id = ? WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(group_id, user_id, app_id)).await?;

	Ok(())
}

pub(super) async fn register(app_id: AppId, register_data: UserDeviceRegisterInput) -> AppRes<(UserId, DeviceId)>
{
	//check first if the user identifier is available
	let check = check_user_exists(app_id.as_str(), register_data.device_identifier.as_str()).await?;

	if check {
		//check true == user exists
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::UserExists,
			"User already exists".to_string(),
			None,
		));
	}

	//data for the user table
	//language=SQL
	let sql_user = "INSERT INTO sentc_user (id, app_id, user_group_id, time) VALUES (?,?,?,?)";
	let user_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//insert a fake group id for now, and update the user group id when user group was created
	let user_params = set_params!(
		user_id.to_string(),
		app_id.to_string(),
		"".to_string(),
		time.to_string()
	);

	let master_key_info = register_data.master_key;
	let derived_data = register_data.derived;

	let device_id = Uuid::new_v4().to_string();

	//data for the user key table
	let (sql_keys, key_params) = prepare_register_device(
		device_id.to_string(),
		user_id.to_string(),
		app_id,
		time,
		register_data.device_identifier,
		master_key_info,
		derived_data,
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

pub(super) async fn register_device(app_id: AppId, input: UserDeviceRegisterInput, token: String) -> AppRes<DeviceId>
{
	let master_key_info = input.master_key;
	let derived_data = input.derived;

	let device_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	let (sql_keys, key_params) = prepare_register_device(
		device_id.to_string(),
		"not_registered".to_string(),
		app_id,
		time,
		input.device_identifier,
		master_key_info,
		derived_data,
		Some(token),
	);

	exec(sql_keys, key_params).await?;

	Ok(device_id)
}

pub(super) async fn get_done_register_device(app_id: AppId, token: String) -> AppRes<DeviceId>
{
	if token.as_str() == "NULL" || token.as_str() == "null" {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::UserNotFound,
			"Device was not found for this token".to_string(),
			None,
		));
	}

	//language=SQL
	let sql = "SELECT id FROM sentc_user_device WHERE app_id = ? AND token = ?";

	let out: Option<UserResetPwCheck> = query_first(sql, set_params!(app_id, token)).await?;

	let device_id: DeviceId = match out {
		Some(id) => id.0,
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::UserNotFound,
				"Device was not found for this token".to_string(),
				None,
			))
		},
	};

	Ok(device_id)
}

pub(super) async fn done_register_device(app_id: AppId, user_id: UserId, device_id: DeviceId) -> AppRes<()>
{
	//update the user id, in two fn because there can be an err for inserting user keys in user group
	//language=SQL
	let sql = "UPDATE sentc_user_device SET user_id = ?, token = NULL WHERE id = ? AND app_id = ?";
	exec(sql, set_params!(user_id, device_id, app_id)).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn delete(user_id: String, app_id: AppId) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_user WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(user_id, app_id)).await?;

	Ok(())
}

pub(super) async fn delete_device(user_id: UserId, app_id: AppId, device_id: DeviceId) -> AppRes<()>
{
	//first check if it is the only device
	//language=SQL
	let sql = "SELECT COUNT(id) FROM sentc_user_device WHERE app_id = ? AND user_id = ? LIMIT 2";

	let device_count: Option<UserExistsEntity> = query_first(sql, set_params!(app_id.to_string(), user_id.to_string())).await?;

	match device_count {
		Some(c) => {
			if c.0 < 2 {
				return Err(HttpErr::new(
					400,
					ApiErrorCodes::UserDeviceDelete,
					"Can't delete the last device. Use user delete instead.".to_string(),
					None,
				));
			}
		},
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::UserDeviceNotFound,
				"No device found".to_string(),
				None,
			));
		},
	}

	//language=SQL
	let sql = "DELETE FROM sentc_user_device WHERE user_id = ? AND id = ? AND app_id = ?";

	exec(sql, set_params!(user_id, device_id, app_id)).await?;

	Ok(())
}

pub(super) async fn update(user_id: UserId, device_id: DeviceId, app_id: AppId, user_identifier: String) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_user_device SET device_identifier = ? WHERE id = ? AND user_id = ? AND app_id = ?";

	exec(sql, set_params!(user_identifier, device_id, user_id, app_id)).await?;

	Ok(())
}

pub(super) async fn change_password(user_id: UserId, device_id: DeviceId, data: ChangePasswordData, old_hashed_auth_key: String) -> AppRes<()>
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
			user_id,
			device_id,
			old_hashed_auth_key
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn reset_password(user_id: UserId, device_id: DeviceId, data: ResetPasswordData) -> AppRes<()>
{
	//reset only the actual device
	//create a new master key from the new password. the key pairs are still the same

	//get the first row (the key id) which we are updating

	//language=SQL
	let sql = "SELECT app_id FROM sentc_user_device WHERE id = ? ORDER BY time DESC LIMIT 1";

	let row: Option<UserResetPwCheck> = query_first(sql, set_params!(device_id.to_string())).await?;

	let row = match row {
		Some(r) => r,
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::UserNotFound,
				"No keys to update".to_string(),
				None,
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
			user_id.to_string(),
			row.0
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn get_devices(app_id: AppId, user_id: UserId, last_fetched_time: u128, last_fetched_id: DeviceId) -> AppRes<Vec<UserDeviceList>>
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
				app_id,
				user_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_id
			),
		)
	} else {
		let sql = sql + " ORDER BY time, id LIMIT 50";

		(sql, set_params!(app_id, user_id))
	};

	let list: Vec<UserDeviceList> = query_string(sql, params).await?;

	Ok(list)
}

pub(super) async fn prepare_user_key_rotation(app_id: AppId, user_id: UserId) -> AppRes<Option<UserResetPwCheck>>
{
	//language=SQL
	let sql = "SELECT user_group_id FROM sentc_user WHERE app_id = ? AND id = ?";

	let check: Option<UserResetPwCheck> = query_first(sql, set_params!(app_id, user_id)).await?;

	Ok(check)
}

//__________________________________________________________________________________________________

pub(super) async fn save_user_action(app_id: AppId, user_id: UserId, action: UserAction, amount: i64) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_user_action_log (user_id, time, action_id, app_id, amount) VALUES (?,?,?,?,?)";

	let action = match action {
		UserAction::Login => 0,
		UserAction::Refresh => 1,
		UserAction::ChangePassword => 2,
		UserAction::ResetPassword => 3,
		UserAction::Delete => 4,
		UserAction::Init => 5,
		UserAction::KeyRotation => 6,
	};

	exec(sql, set_params!(user_id, time.to_string(), action, app_id, amount)).await?;

	Ok(())
}

//__________________________________________________________________________________________________

fn prepare_register_device(
	device_id: DeviceId,
	user_id: UserId,
	app_id: AppId,
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
		device_id.to_string(),
		user_id,
		app_id,
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

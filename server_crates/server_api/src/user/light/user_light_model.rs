use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::db::{exec, exec_transaction, TransactionData};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::{get_time, set_params};
use sentc_crypto_common::user::{KeyDerivedData, MasterKey};
use sentc_crypto_common::{AppId, DeviceId, UserId};

use crate::user::user_model::{check_user_exists, prepare_register_device};
use crate::util::api_res::ApiErrorCodes;

pub(super) async fn register_light(
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

pub(super) async fn reset_password_light(
	app_id: impl Into<AppId>,
	device_identifier: String,
	master_key_info: MasterKey,
	derived_data: KeyDerivedData,
) -> AppRes<()>
{
	//change here also the public keys.

	//language=SQL
	let sql = r"
UPDATE sentc_user_device 
SET 
    client_random_value = ?, 
    public_key = ?, 
    encrypted_private_key = ?, 
    keypair_encrypt_alg = ?, 
    encrypted_sign_key = ?, 
    verify_key = ?,
    keypair_sign_alg = ?, 
    derived_alg = ?, 
    encrypted_master_key = ?, 
    master_key_alg = ?, 
    encrypted_master_key_alg = ?, 
    hashed_auth_key = ?
WHERE app_id = ? AND device_identifier = ?
";

	exec(
		sql,
		set_params!(
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
			app_id.into(),
			device_identifier
		),
	)
	.await
}

//__________________________________________________________________________________________________

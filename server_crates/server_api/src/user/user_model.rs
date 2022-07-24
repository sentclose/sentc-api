use sentc_crypto_common::user::RegisterData;
use uuid::Uuid;

use crate::core::api_res::{ApiErrorCodes, HttpErr};
use crate::core::db::{exec, exec_transaction, query_first, TransactionData};
use crate::core::get_time;
use crate::set_params;
use crate::user::user_entities::{JwtSignKey, JwtVerifyKey, UserEntity, UserExistsEntity};

pub(super) async fn get_jwt_sign_key(kid: &str) -> Result<String, HttpErr>
{
	//language=SQL
	let sql = "SELECT sign_key FROM app_jwt_keys WHERE id = ?";

	let sign_key: Option<JwtSignKey> = query_first(sql.to_string(), set_params!(kid.to_string())).await?;

	match sign_key {
		Some(k) => Ok(k.0),
		None => {
			Err(HttpErr::new(
				200,
				ApiErrorCodes::JwtKeyNotFound,
				"No matched key to this key id",
				None,
			))
		},
	}
}

pub(super) async fn get_jwt_verify_key(kid: &str) -> Result<String, HttpErr>
{
	//language=SQL
	let sql = "SELECT verify_key FROM app_jwt_keys WHERE id = ?";

	let sign_key: Option<JwtVerifyKey> = query_first(sql.to_string(), set_params!(kid.to_string())).await?;

	match sign_key {
		Some(k) => Ok(k.0),
		None => {
			Err(HttpErr::new(
				200,
				ApiErrorCodes::JwtKeyNotFound,
				"No matched key to this key id",
				None,
			))
		},
	}
}

//__________________________________________________________________________________________________
//user

pub(super) async fn check_user_exists(user_identifier: &str) -> Result<bool, HttpErr>
{
	//language=SQL
	let sql = "SELECT 1 FROM user WHERE identifier = ? LIMIT 1";

	let exists: Option<UserExistsEntity> = query_first(sql.to_string(), set_params!(user_identifier.to_string())).await?;

	match exists {
		Some(_) => Ok(true),
		None => Ok(false),
	}
}

pub(super) async fn register(app_id: &str, register_data: RegisterData) -> Result<String, HttpErr>
{
	//check first if the user identifier is available
	let check = check_user_exists(register_data.user_identifier.as_str()).await?;

	if check {
		//check true == user exists
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::UserExists,
			"User already exists",
			None,
		));
	}

	//data for the user table
	//language=SQL
	let sql_user = "INSERT INTO user (id, app_id, identifier, time) VALUES (?,?,?,?)";
	let user_id = Uuid::new_v4().to_string();
	let time = get_time()?;
	let user_params = set_params!(
		user_id.to_string(),
		app_id.to_string(),
		register_data.user_identifier,
		time.to_string()
	);

	//data for the user key table
	//language=SQL
	let sql_keys = r"
INSERT INTO user_keys 
    (id, 
     user_id, 
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
     hashed_auth_key, 
     time
     ) 
VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

	let master_key_info = register_data.master_key;
	let derived_data = register_data.derived;

	let key_id = Uuid::new_v4().to_string();
	let key_params = set_params!(
		key_id,
		user_id.to_string(),
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
		derived_data.hashed_authentication_key,
		time.to_string()
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

	Ok(user_id)
}

pub(super) async fn delete(user_id: &str) -> Result<(), HttpErr>
{
	//language=SQL
	let sql = "DELETE FROM user WHERE id = ?";

	exec(sql, set_params!(user_id.to_owned())).await?;

	Ok(())
}

pub(super) async fn get_user(user_id: &str) -> Result<UserEntity, HttpErr>
{
	//language=SQL
	let sql = "SELECT * FROM test WHERE id = ?";

	let user: Option<UserEntity> = query_first(sql.to_string(), set_params!(user_id.to_string())).await?;

	match user {
		Some(u) => Ok(u),
		None => Err(HttpErr::new(200, ApiErrorCodes::UserNotFound, "user not found", None)),
	}
}

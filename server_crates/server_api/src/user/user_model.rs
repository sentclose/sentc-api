use crate::core::api_err::{ApiErrorCodes, HttpErr};
use crate::core::db::query_first;
use crate::set_params;
use crate::user::user_entities::{JwtSignKey, JwtVerifyKey, UserEntity, UserExistsEntity};

pub(crate) async fn get_jwt_sign_key(kid: &str) -> Result<String, HttpErr>
{
	//language=SQL
	let sql = "SELECT sign_key FROM jwt_key WHERE kid = ?";

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

pub(crate) async fn get_jwt_verify_key(kid: &str) -> Result<String, HttpErr>
{
	//language=SQL
	let sql = "SELECT verify_key FROM jwt_key WHERE kid = ?";

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

pub(crate) async fn check_user_exists(user_id: &str) -> Result<bool, HttpErr>
{
	//language=SQL
	let sql = "SELECT 1 FROM test WHERE id = ? LIMIT 1";

	let exists: Option<UserExistsEntity> = query_first(sql.to_string(), set_params!(user_id.to_string())).await?;

	match exists {
		Some(_) => Ok(true),
		None => Ok(false),
	}
}

pub(crate) async fn get_user(user_id: &str) -> Result<UserEntity, HttpErr>
{
	//language=SQL
	let sql = "SELECT * FROM test WHERE id = ?";

	let user: Option<UserEntity> = query_first(sql.to_string(), set_params!(user_id.to_string())).await?;

	match user {
		Some(u) => Ok(u),
		None => Err(HttpErr::new(200, ApiErrorCodes::UserNotFound, "user not found", None)),
	}
}

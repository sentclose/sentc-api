use sentc_crypto_common::{AppId, CustomerId, SignKeyPairId};
use serde::{Deserialize, Serialize};

use crate::take_or_err;

/**
Data which is used to identify the customers app requests.

Cache this data:
- the valid jwt keys
- the general app data

Only internal values from the db
 */
#[derive(Serialize, Deserialize)]
pub struct AppData
{
	pub app_data: AppDataGeneral,
	pub jwt_data: Vec<AppJwt>, //use the newest jwt data to create a jwt, but use the old one to validate the old jwt.
	pub auth_with_token: AuthWithToken,
	pub options: AppOptionsEntity,
}

/**
Describe what token was sent from the req, the public or private
*/
#[derive(Serialize, Deserialize)]
pub enum AuthWithToken
{
	Public,
	Secret,
}

//__________________________________________________________________________________________________

/**
This values can only be exists once

Only internal values from the db
 */
#[derive(Serialize, Deserialize)]
pub struct AppDataGeneral
{
	pub customer_id: CustomerId,
	pub app_id: AppId,
	pub hashed_secret_token: String,
	pub hashed_public_token: String,
	pub hash_alg: String,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for AppDataGeneral
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			app_id: take_or_err!(row, 0, String),
			customer_id: take_or_err!(row, 1, String),
			hashed_secret_token: take_or_err!(row, 2, String),
			hashed_public_token: take_or_err!(row, 3, String),
			hash_alg: take_or_err!(row, 4, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for AppDataGeneral
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			app_id: take_or_err!(row, 0),
			customer_id: take_or_err!(row, 1),
			hashed_secret_token: take_or_err!(row, 2),
			hashed_public_token: take_or_err!(row, 3),
			hash_alg: take_or_err!(row, 4),
		})
	}
}

//__________________________________________________________________________________________________

/**
The key data for creating jwt for the app.

When customer logged in in the dashboard, sentc internal keys are used.

It is possible to have multiple valid jwt keys.

Only internal values from the db
 */
#[derive(Serialize, Deserialize)]
pub struct AppJwt
{
	pub jwt_key_id: SignKeyPairId,
	pub jwt_alg: String, //should be ES384 for now
	pub time: u128,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for AppJwt
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			jwt_key_id: take_or_err!(row, 0, String),
			jwt_alg: take_or_err!(row, 1, String),
			time: take_or_err!(row, 2, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for AppJwt
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let time: String = take_or_err!(row, 2);
		let time: u128 = time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			jwt_key_id: take_or_err!(row, 0),
			jwt_alg: take_or_err!(row, 1),
			time,
		})
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct AppOptionsEntity
{
	pub group_create: i32,
	pub group_get: i32,
	pub group_invite: i32,
	pub group_reject_invite: i32,
	pub group_accept_invite: i32,

	pub group_join_req: i32,
	pub group_accept_join_req: i32,
	pub group_reject_join_req: i32,

	pub group_key_rotation: i32,

	pub group_user_delete: i32,
	pub group_delete: i32,

	pub group_leave: i32,
	pub group_change_rank: i32,

	pub user_exists: i32,
	pub user_register: i32,
	pub user_delete: i32,
	pub user_update: i32,
	pub user_change_password: i32,
	pub user_reset_password: i32,
	pub user_prepare_login: i32,
	pub user_done_login: i32,
	pub user_public_data: i32,
	pub user_jwt_refresh: i32,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for AppOptionsEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			group_create: take_or_err!(row, 0, i32),
			group_get: take_or_err!(row, 1, i32),

			group_invite: take_or_err!(row, 2, i32),
			group_reject_invite: take_or_err!(row, 3, i32),
			group_accept_invite: take_or_err!(row, 4, i32),

			group_join_req: take_or_err!(row, 5, i32),
			group_accept_join_req: take_or_err!(row, 6, i32),
			group_reject_join_req: take_or_err!(row, 7, i32),

			group_key_rotation: take_or_err!(row, 8, i32),

			group_user_delete: take_or_err!(row, 9, i32),

			group_delete: take_or_err!(row, 10, i32),

			group_leave: take_or_err!(row, 11, i32),
			group_change_rank: take_or_err!(row, 12, i32),

			user_exists: take_or_err!(row, 13, i32),
			user_register: take_or_err!(row, 14, i32),
			user_delete: take_or_err!(row, 15, i32),
			user_update: take_or_err!(row, 16, i32),
			user_change_password: take_or_err!(row, 17, i32),
			user_reset_password: take_or_err!(row, 18, i32),
			user_prepare_login: take_or_err!(row, 19, i32),
			user_done_login: take_or_err!(row, 20, i32),
			user_public_data: take_or_err!(row, 21, i32),
			user_jwt_refresh: take_or_err!(row, 22, i32),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for AppOptionsEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			group_create: take_or_err!(row, 0),
			group_get: take_or_err!(row, 1),

			group_invite: take_or_err!(row, 2),
			group_reject_invite: take_or_err!(row, 3),
			group_accept_invite: take_or_err!(row, 4),

			group_join_req: take_or_err!(row, 5),
			group_accept_join_req: take_or_err!(row, 6),
			group_reject_join_req: take_or_err!(row, 7),

			group_key_rotation: take_or_err!(row, 8),

			group_user_delete: take_or_err!(row, 9),

			group_delete: take_or_err!(row, 10),

			group_leave: take_or_err!(row, 11),
			group_change_rank: take_or_err!(row, 12),

			user_exists: take_or_err!(row, 13),
			user_register: take_or_err!(row, 14),
			user_delete: take_or_err!(row, 15),
			user_update: take_or_err!(row, 16),
			user_change_password: take_or_err!(row, 17),
			user_reset_password: take_or_err!(row, 18),
			user_prepare_login: take_or_err!(row, 19),
			user_done_login: take_or_err!(row, 20),
			user_public_data: take_or_err!(row, 21),
			user_jwt_refresh: take_or_err!(row, 22),
		})
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub(crate) struct AppExistsEntity(pub i64);

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for AppExistsEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0, i64)))
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for AppExistsEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0)))
	}
}

//__________________________________________________________________________________________________

//copy in api common crate but without the db trait impl
#[derive(Serialize, Deserialize)]
pub struct AppJwtData
{
	pub jwt_key_id: SignKeyPairId,
	pub jwt_alg: String, //should be ES384 for now
	pub time: u128,
	pub sign_key: String,
	pub verify_key: String,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for AppJwtData
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			jwt_key_id: take_or_err!(row, 0, String),
			jwt_alg: take_or_err!(row, 1, String),
			time: take_or_err!(row, 2, u128),
			sign_key: take_or_err!(row, 3, String),
			verify_key: take_or_err!(row, 4, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for AppJwtData
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let time: String = take_or_err!(row, 2);
		let time: u128 = time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			jwt_key_id: take_or_err!(row, 0),
			jwt_alg: take_or_err!(row, 1),
			time,
			sign_key: take_or_err!(row, 3),
			verify_key: take_or_err!(row, 4),
		})
	}
}

//__________________________________________________________________________________________________

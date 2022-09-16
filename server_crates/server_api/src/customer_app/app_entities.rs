use sentc_crypto_common::{AppId, CustomerId, SignKeyPairId};
use serde::{Deserialize, Serialize};
use server_api_common::app::AppOptions;
use server_core::take_or_err;

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
	pub options: AppOptions,
	pub file_options: AppFileOptions,
}

#[derive(Serialize, Deserialize)]
pub struct AppFileOptions
{
	pub file_storage: i32,
	pub storage_url: Option<String>,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for AppFileOptions
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			file_storage: take_or_err!(row, 0, i32),
			storage_url: server_core::take_or_err_opt!(row, 1, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for AppFileOptions
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			file_storage: take_or_err!(row, 0),
			storage_url: take_or_err!(row, 1),
		})
	}
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
impl server_core::db::FromSqliteRow for AppDataGeneral
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
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
impl server_core::db::FromSqliteRow for AppJwt
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			jwt_key_id: take_or_err!(row, 0),
			jwt_alg: take_or_err!(row, 1),
			time: server_core::take_or_err_u128!(row, 2),
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
impl server_core::db::FromSqliteRow for AppExistsEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0)))
	}
}

//__________________________________________________________________________________________________

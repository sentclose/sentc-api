use sentc_crypto_common::{AppId, CustomerId, SignKeyPairId};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

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
pub struct AppRegisterInput
{
	pub identifier: Option<String>,
}

impl AppRegisterInput
{
	pub fn to_string(&self) -> serde_json::Result<String>
	{
		to_string(self)
	}
}

/**
When creating multiple jwt keys for this app

Always return this for every new jwt key pair
 */
#[derive(Serialize, Deserialize)]
pub struct AppJwtRegisterOutput
{
	pub customer_id: CustomerId,
	pub app_id: AppId,
	pub jwt_verify_key: String,
	pub jwt_sign_key: String,
	pub jwt_alg: String, //should be ES384 for now
}

#[derive(Serialize, Deserialize)]
pub struct AppRegisterOutput
{
	pub customer_id: CustomerId,
	pub app_id: AppId,

	//don't show this values in te normal app data
	pub secret_token: String,
	pub public_token: String,

	pub jwt_data: AppJwtRegisterOutput,
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct AppTokenRenewOutput
{
	pub secret_token: String,
	pub public_token: String,
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct AppDeleteOutput
{
	pub old_app_id: AppId,
	pub msg: String,
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct AppUpdateOutput
{
	pub app_id: AppId,
	pub msg: String,
}

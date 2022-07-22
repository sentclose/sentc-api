use sentc_crypto_common::UserId;
use serde::{Deserialize, Serialize};

use crate::take_or_err;

#[derive(Serialize, Deserialize)]
pub struct UserJwtEntity
{
	pub id: UserId,
	pub identifier: String,
	//aud if it is an app user or an customer
	pub aud: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserEntity
{
	id: String,
	name: String,
	time: u128,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(UserEntity {
			id: take_or_err!(row, 0, String),
			name: take_or_err!(row, 1, String),
			time: take_or_err!(row, 2, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for UserEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		//time needs to parse from string to the value
		let time: String = take_or_err!(row, 2);
		let time: u128 = time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(UserEntity {
			id: take_or_err!(row, 0),
			name: take_or_err!(row, 1),
			time: time,
		})
	}
}

#[derive(Serialize, Deserialize)]
pub struct UserExistsEntity(pub i64); //i64 for sqlite

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserExistsEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(UserExistsEntity(take_or_err!(row, 0, i64)))
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for UserExistsEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(UserExistsEntity(take_or_err!(row, 0)))
	}
}

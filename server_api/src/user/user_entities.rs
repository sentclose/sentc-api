use crate::take_or_err;

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

pub struct UserExistsEntity(pub i64); //i64 for sqlite

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserExistsEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(UserExistsEntity(take_or_err!(row, 2, i64)))
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

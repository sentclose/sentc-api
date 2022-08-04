use crate::take_or_err;

#[cfg(feature = "send_mail")]
pub(crate) enum RegisterEmailStatus
{
	Success,
	FailedMessage(String),
	FailedSend(String),
	Other(String),
}

//__________________________________________________________________________________________________

pub(crate) struct CustomerEmailValid(pub i32);

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for CustomerEmailValid
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0, i32)))
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for CustomerEmailValid
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0)))
	}
}

//__________________________________________________________________________________________________

pub(crate) struct CustomerDataEntity
{
	pub email: String,
	pub email_valid: i32,
	pub email_send: u128,
	pub email_status: i32,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for CustomerDataEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			email: take_or_err!(row, 0, String),
			email_valid: take_or_err!(row, 1, i32),
			email_send: take_or_err!(row, 2, u128),
			email_status: take_or_err!(row, 3, i32),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for CustomerDataEntity
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
			email: take_or_err!(row, 0),
			email_valid: take_or_err!(row, 1),
			email_send: time,
			email_status: take_or_err!(row, 3),
		})
	}
}

//__________________________________________________________________________________________________

pub(crate) struct CustomerEmailToken(pub String);

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for CustomerEmailToken
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0, String)))
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for CustomerEmailToken
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0)))
	}
}

//__________________________________________________________________________________________________

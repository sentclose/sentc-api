#![allow(dead_code)]

use sentc_crypto_common::{CustomerId, DeviceId};
use server_core::take_or_err;

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
impl server_core::db::FromSqliteRow for CustomerEmailValid
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
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
impl server_core::db::FromSqliteRow for CustomerDataEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			email: take_or_err!(row, 0),
			email_valid: take_or_err!(row, 1),
			email_send: server_core::take_or_err_u128!(row, 2),
			email_status: take_or_err!(row, 3),
		})
	}
}

//__________________________________________________________________________________________________

pub(crate) struct CustomerDataByEmailEntity
{
	pub id: CustomerId,
	pub email_valid: i32,
	pub email_send: u128,
	pub email_status: i32,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for CustomerDataByEmailEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			id: take_or_err!(row, 0, String),
			email_valid: take_or_err!(row, 1, i32),
			email_send: take_or_err!(row, 2, u128),
			email_status: take_or_err!(row, 3, i32),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for CustomerDataByEmailEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			id: take_or_err!(row, 0),
			email_valid: take_or_err!(row, 1),
			email_send: server_core::take_or_err_u128!(row, 2),
			email_status: take_or_err!(row, 3),
		})
	}
}

//__________________________________________________________________________________________________

pub(crate) struct CustomerEmailToken
{
	pub email_token: String,
	pub email: String,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for CustomerEmailToken
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			email_token: take_or_err!(row, 0, String),
			email: take_or_err!(row, 1, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for CustomerEmailToken
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			email_token: take_or_err!(row, 0),
			email: take_or_err!(row, 1),
		})
	}
}

//__________________________________________________________________________________________________

pub(crate) struct CustomerEmailByToken
{
	pub email: String,
	pub id: CustomerId,
	pub device_id: DeviceId,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for CustomerEmailByToken
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			email: take_or_err!(row, 0, String),
			id: take_or_err!(row, 1, String),
			device_id: take_or_err!(row, 2, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for CustomerEmailByToken
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			email: take_or_err!(row, 0),
			id: take_or_err!(row, 1),
			device_id: take_or_err!(row, 2),
		})
	}
}

//__________________________________________________________________________________________________

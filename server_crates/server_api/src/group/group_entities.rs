use crate::take_or_err;

/**
invite (keys needed)
*/
pub static GROUP_INVITE_TYPE_INVITE_REQ: u16 = 0;

/**
join req (no keys needed)
*/
pub static GROUP_INVITE_TYPE_JOIN_REQ: u16 = 1;

//__________________________________________________________________________________________________

pub struct UserGroupRankCheck(pub i32);

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserGroupRankCheck
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0, i32)))
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for UserGroupRankCheck
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0)))
	}
}

//__________________________________________________________________________________________________

pub struct UserInGroupCheck(pub i32);

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserInGroupCheck
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0, i32)))
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for UserInGroupCheck
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0)))
	}
}

//__________________________________________________________________________________________________

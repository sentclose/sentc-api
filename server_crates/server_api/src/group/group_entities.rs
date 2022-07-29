use sentc_crypto_common::{AppId, GroupId, UserId};
use serde::{Deserialize, Serialize};

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

/**
internally used in cache to check every group
 */
#[derive(Serialize, Deserialize)]
pub struct InternalGroupData
{
	pub group_id: GroupId,
	pub app_id: AppId,
	pub parent: Option<GroupId>,
	pub time: u128,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for InternalGroupData
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		//use this here because the macro don't handle Option<T>
		let parent = match row.take_opt::<Option<String>, _>(2) {
			Some(value) => {
				match value {
					Ok(ir) => ir,
					Err(mysql_async::FromValueError(_value)) => {
						return Err(mysql_async::FromRowError(row));
					},
				}
			},
			None => return Err(mysql_async::FromRowError(row)),
		};

		Ok(Self {
			group_id: take_or_err!(row, 0, String),
			app_id: take_or_err!(row, 1, String),
			parent,
			time: take_or_err!(row, 3, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for InternalGroupData
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let time: String = take_or_err!(row, 3);
		let time: u128 = time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			group_id: take_or_err!(row, 0),
			app_id: take_or_err!(row, 1),
			parent: take_or_err!(row, 2),
			time,
		})
	}
}

//__________________________________________________________________________________________________

/**
internally used in cache to check every user
*/
#[derive(Serialize, Deserialize)]
pub struct InternalUserGroupData
{
	pub user_id: UserId,
	pub joined_time: u128,
	pub rank: i32,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for InternalUserGroupData
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			user_id: take_or_err!(row, 0, String),
			joined_time: take_or_err!(row, 1, u128),
			rank: take_or_err!(row, 2, i32),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for InternalUserGroupData
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let time: String = take_or_err!(row, 1);
		let time: u128 = time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			user_id: take_or_err!(row, 0),
			joined_time: time,
			rank: take_or_err!(row, 2),
		})
	}
}

//__________________________________________________________________________________________________

/**
internally used in cache to check every user and group
 */
pub struct InternalGroupDataComplete
{
	pub group_data: InternalGroupData,
	pub user_data: InternalUserGroupData,
}

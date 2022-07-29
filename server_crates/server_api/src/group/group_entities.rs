use sentc_crypto_common::group::GroupKeyServerOutput;
use sentc_crypto_common::{GroupId, SymKeyId};

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

pub struct GroupKeyUpdate
{
	pub encrypted_ephemeral_key: String,
	pub encrypted_eph_key_key_id: String,
	pub encrypted_group_key_by_eph_key: String,
	pub previous_group_key_id: SymKeyId,
	pub time: u128,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for GroupKeyUpdate
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			encrypted_ephemeral_key: take_or_err!(row, 0, String),
			encrypted_eph_key_key_id: take_or_err!(row, 1, String),
			encrypted_group_key_by_eph_key: take_or_err!(row, 2, String),
			previous_group_key_id: take_or_err!(row, 3, String),
			time: take_or_err!(row, 4, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for GroupKeyUpdate
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let time: String = take_or_err!(row, 4);
		let time: u128 = time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			encrypted_ephemeral_key: take_or_err!(row, 0),
			encrypted_eph_key_key_id: take_or_err!(row, 1),
			encrypted_group_key_by_eph_key: take_or_err!(row, 2),
			previous_group_key_id: take_or_err!(row, 3),
			time,
		})
	}
}

//__________________________________________________________________________________________________

pub struct GroupKeyUpdateReady(pub i32);

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for GroupKeyUpdateReady
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0, i32)))
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for GroupKeyUpdateReady
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0)))
	}
}

//__________________________________________________________________________________________________

pub struct GroupUserData
{
	pub id: GroupId,
	pub parent_group_id: Option<GroupId>,
	pub rank: i32,
	pub created_time: u128,
	pub joined_time: u128,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for GroupUserData
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		//use this here because the macro don't handle Option<T>
		let parent_group_id = match row.take_opt::<Option<String>, _>(1) {
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
			id: take_or_err!(row, 0, String),
			parent_group_id,
			rank: take_or_err!(row, 2, i32),
			created_time: take_or_err!(row, 3, u128),
			joined_time: take_or_err!(row, 4, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for GroupUserData
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		//time needs to parse from string to the value
		let created_time: String = take_or_err!(row, 3);
		let created_time: u128 = created_time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		let joined_time: String = take_or_err!(row, 4);
		let joined_time: u128 = joined_time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			id: take_or_err!(row, 0),
			parent_group_id: take_or_err!(row, 1),
			rank: take_or_err!(row, 2),
			created_time,
			joined_time,
		})
	}
}

//__________________________________________________________________________________________________

pub struct GroupUserKeys
{
	pub k_id: String,
	pub encrypted_group_key: String,
	pub group_key_alg: String,
	pub encrypted_private_key: String,
	pub public_key: String,
	pub private_key_pair_alg: String,
	pub encrypted_group_key_key_id: String,
	//pub time: u128,
}

impl Into<GroupKeyServerOutput> for GroupUserKeys
{
	fn into(self) -> GroupKeyServerOutput
	{
		GroupKeyServerOutput {
			encrypted_group_key: self.encrypted_group_key,
			group_key_alg: self.group_key_alg,
			group_key_id: self.k_id.to_string(),
			encrypted_private_group_key: self.encrypted_private_key,
			public_group_key: self.public_key,
			keypair_encrypt_alg: self.private_key_pair_alg,
			key_pair_id: self.k_id,
			user_public_key_id: self.encrypted_group_key_key_id,
		}
	}
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for GroupUserKeys
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			k_id: take_or_err!(row, 0, String),
			encrypted_group_key: take_or_err!(row, 1, String),
			group_key_alg: take_or_err!(row, 2, String),
			encrypted_private_key: take_or_err!(row, 3, String),
			public_key: take_or_err!(row, 4, String),
			private_key_pair_alg: take_or_err!(row, 5, String),
			encrypted_group_key_key_id: take_or_err!(row, 6, String),
			//time: take_or_err!(row, 7, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for GroupUserKeys
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		/*
		//time needs to parse from string to the value
		let time: String = take_or_err!(row, 7);
		let time: u128 = time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;
		 */

		Ok(Self {
			k_id: take_or_err!(row, 0),
			encrypted_group_key: take_or_err!(row, 1),
			group_key_alg: take_or_err!(row, 2),
			encrypted_private_key: take_or_err!(row, 3),
			public_key: take_or_err!(row, 4),
			private_key_pair_alg: take_or_err!(row, 5),
			encrypted_group_key_key_id: take_or_err!(row, 6),
			//time,
		})
	}
}

//__________________________________________________________________________________________________

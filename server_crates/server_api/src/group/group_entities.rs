use sentc_crypto_common::group::{GroupInviteReqList, GroupJoinReqList, GroupKeyServerOutput, KeyRotationInput};
use sentc_crypto_common::{AppId, EncryptionKeyPairId, GroupId, SymKeyId, UserId};
use serde::{Deserialize, Serialize};

use crate::take_or_err;

pub type GroupNewUserType = u16;

/**
invite (keys needed)
*/
pub static GROUP_INVITE_TYPE_INVITE_REQ: GroupNewUserType = 0;

/**
join req (no keys needed)
*/
pub static GROUP_INVITE_TYPE_JOIN_REQ: GroupNewUserType = 1;

//__________________________________________________________________________________________________

/**
Internal used group data, to check if the group exists with this app id
*/
#[derive(Serialize, Deserialize)]
pub struct InternalGroupData
{
	pub app_id: AppId,
	pub id: GroupId,
	pub time: u128,
	pub parent: Option<GroupId>,
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
			id: take_or_err!(row, 0, String),
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
			id: take_or_err!(row, 0),
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
	pub user_id: UserId,      //can be the parent group id (which is a user in this case)
	pub real_user_id: UserId, //the real user
	pub joined_time: u128,
	pub rank: i32,
	pub get_values_from_parent: Option<GroupId>, //if the user is in a parent group -> get the user data of this parent to get the rank
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for InternalUserGroupData
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		let user_id = take_or_err!(row, 0, String);

		Ok(Self {
			real_user_id: user_id.to_string(),
			user_id,
			joined_time: take_or_err!(row, 1, u128),
			rank: take_or_err!(row, 2, i32),
			get_values_from_parent: None,
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

		let user_id: String = take_or_err!(row, 0);

		Ok(Self {
			real_user_id: user_id.to_string(),
			user_id,
			joined_time: time,
			rank: take_or_err!(row, 2),
			get_values_from_parent: None,
		})
	}
}

//__________________________________________________________________________________________________

/**
internally used in cache to check every user

This is fetched when the user is not a direct member but a member from a parent.
 */
#[derive(Serialize, Deserialize)]
pub struct InternalUserGroupDataFromParent
{
	pub get_values_from_parent: GroupId,
	pub joined_time: u128,
	pub rank: i32,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for InternalUserGroupDataFromParent
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			get_values_from_parent: take_or_err!(row, 0, String),
			joined_time: take_or_err!(row, 1, u128),
			rank: take_or_err!(row, 2, i32),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for InternalUserGroupDataFromParent
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
			get_values_from_parent: take_or_err!(row, 0),
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

pub struct GroupChildren(pub GroupId);

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for GroupChildren
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0, String)))
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for GroupChildren
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0)))
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
	pub time: u128,
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
			time: self.time,
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
			time: take_or_err!(row, 7, u128),
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
		//time needs to parse from string to the value
		let time: String = take_or_err!(row, 7);
		let time: u128 = time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			k_id: take_or_err!(row, 0),
			encrypted_group_key: take_or_err!(row, 1),
			group_key_alg: take_or_err!(row, 2),
			encrypted_private_key: take_or_err!(row, 3),
			public_key: take_or_err!(row, 4),
			private_key_pair_alg: take_or_err!(row, 5),
			encrypted_group_key_key_id: take_or_err!(row, 6),
			time,
		})
	}
}

//__________________________________________________________________________________________________

pub struct GroupJoinReq
{
	pub user_id: UserId,
	pub time: u128,
}

impl Into<GroupJoinReqList> for GroupJoinReq
{
	fn into(self) -> GroupJoinReqList
	{
		GroupJoinReqList {
			user_id: self.user_id,
			time: self.time,
		}
	}
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for GroupJoinReq
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			user_id: take_or_err!(row, 0, String),
			time: take_or_err!(row, 1, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for GroupJoinReq
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
			time,
		})
	}
}

//__________________________________________________________________________________________________

pub struct GroupInviteReq
{
	pub group_id: GroupId,
	pub time: u128,
}

impl Into<GroupInviteReqList> for GroupInviteReq
{
	fn into(self) -> GroupInviteReqList
	{
		GroupInviteReqList {
			group_id: self.group_id,
			time: self.time,
		}
	}
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for GroupInviteReq
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			group_id: take_or_err!(row, 0, String),
			time: take_or_err!(row, 1, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for GroupInviteReq
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
			group_id: take_or_err!(row, 0),
			time,
		})
	}
}

//__________________________________________________________________________________________________

pub struct GroupKeyUpdate
{
	pub encrypted_ephemeral_key: String,
	pub encrypted_eph_key_key_id: String,
	pub encrypted_group_key_by_eph_key: EncryptionKeyPairId,
	pub previous_group_key_id: SymKeyId,
	pub ephemeral_alg: String,
	pub time: u128,
	pub new_group_key_id: SymKeyId,
}

impl Into<KeyRotationInput> for GroupKeyUpdate
{
	fn into(self) -> KeyRotationInput
	{
		KeyRotationInput {
			encrypted_ephemeral_key_by_group_key_and_public_key: self.encrypted_ephemeral_key,
			encrypted_group_key_by_ephemeral: self.encrypted_group_key_by_eph_key,
			ephemeral_alg: self.ephemeral_alg,
			previous_group_key_id: self.previous_group_key_id,
			encrypted_eph_key_key_id: self.encrypted_eph_key_key_id,
			time: self.time,
			new_group_key_id: self.new_group_key_id,
		}
	}
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for GroupKeyUpdate
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			new_group_key_id: take_or_err!(row, 0, String),
			encrypted_ephemeral_key: take_or_err!(row, 1, String),
			encrypted_eph_key_key_id: take_or_err!(row, 2, String),
			encrypted_group_key_by_eph_key: take_or_err!(row, 3, String),
			previous_group_key_id: take_or_err!(row, 4, String),
			ephemeral_alg: take_or_err!(row, 5, String),
			time: take_or_err!(row, 6, u128),
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
		let time: String = take_or_err!(row, 6);
		let time: u128 = time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			new_group_key_id: take_or_err!(row, 0),
			encrypted_ephemeral_key: take_or_err!(row, 1),
			encrypted_eph_key_key_id: take_or_err!(row, 2),
			encrypted_group_key_by_eph_key: take_or_err!(row, 3),
			previous_group_key_id: take_or_err!(row, 4),
			ephemeral_alg: take_or_err!(row, 5),
			time,
		})
	}
}

//__________________________________________________________________________________________________

pub struct KeyRotationWorkerKey
{
	pub ephemeral_alg: String,
	pub encrypted_ephemeral_key: String,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for KeyRotationWorkerKey
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			ephemeral_alg: take_or_err!(row, 0, String),
			encrypted_ephemeral_key: take_or_err!(row, 1, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for KeyRotationWorkerKey
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			ephemeral_alg: take_or_err!(row, 0),
			encrypted_ephemeral_key: take_or_err!(row, 1),
		})
	}
}

//__________________________________________________________________________________________________

/**
Output after key rotation for each user
*/
pub struct UserEphKeyOut
{
	pub user_id: UserId,
	pub encrypted_ephemeral_key: String,
	pub encrypted_eph_key_key_id: EncryptionKeyPairId,
}

//__________________________________________________________________________________________________

pub struct UserGroupPublicKeyData
{
	pub user_id: UserId,
	pub public_key_id: EncryptionKeyPairId,
	pub public_key: String,
	pub public_key_alg: String,
	pub time: u128,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserGroupPublicKeyData
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			user_id: take_or_err!(row, 0, String),
			public_key_id: take_or_err!(row, 1, String),
			public_key: take_or_err!(row, 2, String),
			public_key_alg: take_or_err!(row, 3, String),
			time: take_or_err!(row, 4, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for UserGroupPublicKeyData
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
			user_id: take_or_err!(row, 0),
			public_key_id: take_or_err!(row, 1),
			public_key: take_or_err!(row, 2),
			public_key_alg: take_or_err!(row, 3),
			time,
		})
	}
}

//__________________________________________________________________________________________________

pub struct GroupKeySession(pub UserId);

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for GroupKeySession
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0, String)))
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for GroupKeySession
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0)))
	}
}

//__________________________________________________________________________________________________

pub struct GroupUserListItem
{
	pub user_id: UserId,
	pub rank: i32,
	pub joined_time: u128,
}

impl Into<sentc_crypto_common::group::GroupUserListItem> for GroupUserListItem
{
	fn into(self) -> sentc_crypto_common::group::GroupUserListItem
	{
		sentc_crypto_common::group::GroupUserListItem {
			user_id: self.user_id,
			rank: self.rank,
			joined_time: self.joined_time,
		}
	}
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for GroupUserListItem
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			user_id: take_or_err!(row, 0, String),
			rank: take_or_err!(row, 1, i32),
			joined_time: take_or_err!(row, 2, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl crate::core::db::FromSqliteRow for GroupUserListItem
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let joined_time: String = take_or_err!(row, 2);
		let joined_time: u128 = joined_time.parse().map_err(|e| {
			crate::core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			user_id: take_or_err!(row, 0),
			rank: take_or_err!(row, 1),
			joined_time,
		})
	}
}

//__________________________________________________________________________________________________

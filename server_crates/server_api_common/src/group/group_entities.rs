use rustgram_server_util::{take_or_err, DB};
use sentc_crypto_common::{AppId, GroupId, UserId};
use serde::{Deserialize, Serialize};

/**
Internal used group data, to check if the group exists with this app id
 */
#[derive(Serialize, Deserialize, DB)]
pub struct InternalGroupData
{
	pub id: GroupId,
	pub app_id: AppId,
	pub parent: Option<GroupId>,
	pub time: u128,
	pub invite: i32,
	pub is_connected_group: bool,
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
	//if the user enters this group from another group as member (can be from parent too)
	//store the id because the user id can be overwritten by the parent
	pub get_values_from_group_as_member: Option<GroupId>,
}

#[cfg(feature = "mysql")]
impl rustgram_server_util::db::mysql_async_export::prelude::FromRow for InternalUserGroupData
{
	fn from_row_opt(
		mut row: rustgram_server_util::db::mysql_async_export::Row,
	) -> Result<Self, rustgram_server_util::db::mysql_async_export::FromRowError>
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
			get_values_from_group_as_member: None,
		})
	}
}

#[cfg(feature = "sqlite")]
impl rustgram_server_util::db::FromSqliteRow for InternalUserGroupData
{
	fn from_row_opt(row: &rustgram_server_util::db::rusqlite_export::Row) -> Result<Self, rustgram_server_util::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let user_id: String = take_or_err!(row, 0);

		Ok(Self {
			real_user_id: user_id.to_string(),
			user_id,
			joined_time: rustgram_server_util::take_or_err_u128!(row, 1),
			rank: take_or_err!(row, 2),
			get_values_from_parent: None,
			get_values_from_group_as_member: None,
		})
	}
}

//__________________________________________________________________________________________________

/**
internally used in cache to check every user

This is fetched when the user is not a direct member but a member from a parent.
 */
#[derive(Serialize, Deserialize, DB)]
pub struct InternalUserGroupDataFromParent
{
	pub get_values_from_parent: GroupId,
	pub joined_time: u128,
	pub rank: i32,
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

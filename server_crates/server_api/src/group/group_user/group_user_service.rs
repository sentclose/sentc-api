use std::future::Future;

use sentc_crypto_common::group::{GroupKeysForNewMember, GroupKeysForNewMemberServerInput, GroupNewMemberLightInput};
use sentc_crypto_common::{AppId, GroupId, UserId};
use server_core::cache;
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::res::AppRes;

use crate::group::group_entities::{GroupInviteReq, InternalGroupDataComplete};
use crate::group::group_model;
use crate::group::group_user::group_user_model;
use crate::sentc_group_entities::GroupUserListItem;
use crate::util::api_res::ApiErrorCodes;
use crate::util::get_group_user_cache_key;

pub enum InsertNewUserType
{
	Invite,
	Join,
}

pub enum NewUserType
{
	Normal,
	Group,
}

pub fn get_group_member<'a>(
	group_id: impl Into<GroupId> + 'a,
	user_id: impl Into<UserId> + 'a,
	last_fetched_time: u128,
	last_user_id: impl Into<UserId> + 'a,
) -> impl Future<Output = AppRes<Vec<GroupUserListItem>>>
{
	group_user_model::get_group_member(group_id, user_id, last_fetched_time, last_user_id)
}

pub fn get_invite_req<'a>(
	app_id: impl Into<AppId> + 'a,
	user_id: impl Into<UserId> + 'a,
	last_fetched_time: u128,
	last_id: impl Into<GroupId> + 'a,
) -> impl Future<Output = AppRes<Vec<GroupInviteReq>>> + 'a
{
	group_user_model::get_invite_req_to_user(app_id, user_id, last_fetched_time, last_id)
}

pub fn check_is_connected_group<'a>(group_id: impl Into<GroupId> + 'a) -> impl Future<Output = AppRes<i32>> + 'a
{
	group_user_model::check_is_connected_group(group_id)
}

pub async fn invite_request_light(
	group_data: &InternalGroupDataComplete,
	input: GroupNewMemberLightInput,
	invited_user: impl Into<UserId>,
	user_type: NewUserType,
) -> AppRes<()>
{
	if group_data.group_data.invite == 0 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupInviteStop,
			"No invites allowed for this group",
		));
	}

	let rank = input.rank.unwrap_or(4);

	if rank < 1 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserRank,
			"User group rank got the wrong format",
		));
	}

	if rank < group_data.user_data.rank {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserRank,
			"The set rank cannot be higher than your rank",
		));
	}

	group_user_model::invite_request_light(
		&group_data.group_data.id,
		invited_user,
		rank,
		group_data.user_data.rank,
		user_type,
	)
	.await?;

	Ok(())
}

/**
# Group invite request to a non group member user

The invited user must accept the invite to join the group
*/
pub async fn invite_request(
	group_data: &InternalGroupDataComplete,
	input: GroupKeysForNewMemberServerInput,
	invited_user: impl Into<UserId>,
	user_type: NewUserType,
) -> AppRes<Option<String>>
{
	if input.keys.is_empty() {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupNoKeys,
			"No group keys for the user",
		));
	}

	if input.keys.len() > 100 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupTooManyKeys,
			"Too many group keys for the user. Split the keys and use pagination",
		));
	}

	if group_data.group_data.invite == 0 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupInviteStop,
			"No invites allowed for this group",
		));
	}

	let rank = input.rank.unwrap_or(4);

	if rank < 1 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserRank,
			"User group rank got the wrong format",
		));
	}

	if rank < group_data.user_data.rank {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserRank,
			"The set rank cannot be higher than your rank",
		));
	}

	let session_id = group_user_model::invite_request(
		&group_data.group_data.id,
		invited_user,
		input.keys,
		input.key_session,
		rank,
		group_data.user_data.rank,
		user_type,
	)
	.await?;

	Ok(session_id)
}

pub async fn accept_invite(app_id: &str, group_id: impl Into<GroupId>, invited_user: impl Into<UserId>) -> AppRes<()>
{
	let invited_user = invited_user.into();
	let group_id = group_id.into();

	//delete the cache here so the user can join the group
	let key_user = get_group_user_cache_key(app_id, &group_id, &invited_user);
	cache::delete(&key_user).await;

	group_user_model::accept_invite(group_id, invited_user).await?;

	Ok(())
}

pub async fn invite_auto_light(
	group_data: &InternalGroupDataComplete,
	input: GroupNewMemberLightInput,
	invited_user: impl Into<UserId>,
	user_type: NewUserType,
) -> AppRes<()>
{
	let invited_user = invited_user.into();

	invite_request_light(group_data, input, &invited_user, user_type).await?;

	accept_invite(&group_data.group_data.app_id, &group_data.group_data.id, invited_user).await?;

	Ok(())
}

/**
# Invite a non group member user and accept the invite

The first half is the same as invite_request but after accept the invite request without new request
*/
pub async fn invite_auto(
	group_data: &InternalGroupDataComplete,
	input: GroupKeysForNewMemberServerInput,
	invited_user: impl Into<UserId>,
	user_type: NewUserType,
) -> AppRes<Option<String>>
{
	let invited_user = invited_user.into();

	let session_id = invite_request(group_data, input, &invited_user, user_type).await?;

	accept_invite(&group_data.group_data.app_id, &group_data.group_data.id, invited_user).await?;

	Ok(session_id)
}

pub async fn insert_user_keys_via_session(
	group_id: impl Into<GroupId>,
	key_session_id: impl Into<String>,
	insert_type: InsertNewUserType,
	input: Vec<GroupKeysForNewMember>,
) -> AppRes<()>
{
	if input.is_empty() {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupNoKeys,
			"No group keys for the user",
		));
	}

	if input.len() > 100 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupTooManyKeys,
			"Too many group keys for the user. Split the keys and use pagination",
		));
	}

	group_user_model::insert_user_keys_via_session(group_id, key_session_id, input, insert_type).await?;

	Ok(())
}

/**
This fn can be used when there is an error with the user keys e.g. after a key rotation.

The difference to auto invite is that the user must be already in the group and got the same rank etc. back.
*/
pub async fn re_invite_user(
	group_data: &InternalGroupDataComplete,
	mut input: GroupKeysForNewMemberServerInput,
	invited_user: impl Into<UserId>,
	user_type: NewUserType,
) -> AppRes<Option<String>>
{
	let invited_user = invited_user.into();

	//check first if the user or the group is in the group
	let rank = group_user_model::check_user_in_group_direct(&group_data.group_data.id, &invited_user)
		.await?
		.ok_or_else(|| {
			SentcCoreError::new_msg(
				400,
				ApiErrorCodes::GroupReInviteMemberNotFound,
				"User is not in the group. Only group member can be re invited.",
			)
		})?;

	//kick user from the group
	kick_user_from_group(group_data, &invited_user).await?;

	input.rank = Some(rank);

	//and auto invite the user
	invite_auto(group_data, input, invited_user, user_type).await
}

pub async fn leave_group(group_data: &InternalGroupDataComplete, real_user_id: Option<&str>) -> AppRes<()>
{
	if let (Some(g_a_m), Some(id)) = (&group_data.user_data.get_values_from_group_as_member, real_user_id) {
		//if user got access by group as member -> check the rank of the user in the real group.
		// this is important because only group admins can leave a connected group

		//do this check everytime with db look up
		// because the rank in the group data only shows the relative rank in the connected group, not the real rank in the group.
		let group_as_member_user_data = group_model::get_internal_group_user_data(g_a_m, id).await?;

		match group_as_member_user_data {
			Some(data) => {
				group_model::check_group_rank(data.rank, 1)?;
			},
			None => {
				return Err(SentcCoreError::new_msg(
					400,
					ApiErrorCodes::GroupUserRank,
					"Wrong group rank for this action",
				));
			},
		}
	}

	group_user_model::user_leave_group(
		&group_data.group_data.id,
		&group_data.user_data.user_id,
		group_data.user_data.rank,
	)
	.await?;

	let key_group = get_group_user_cache_key(
		group_data.group_data.app_id.as_str(),
		group_data.group_data.id.as_str(),
		group_data.user_data.user_id.as_str(),
	);

	cache::delete(key_group.as_str()).await;

	Ok(())
}

pub async fn kick_user_from_group(group_data: &InternalGroupDataComplete, user_id: impl Into<UserId>) -> AppRes<()>
{
	let user_id = user_id.into();

	//delete the user cache
	let key_group = get_group_user_cache_key(&group_data.group_data.app_id, &group_data.group_data.id, &user_id);
	cache::delete(&key_group).await;

	group_user_model::kick_user_from_group(&group_data.group_data.id, user_id, group_data.user_data.rank).await?;

	Ok(())
}

/**
Update the user rank. The rank of a creator cannot changed.

When deleting the cache for this group, and the group got children then for all children the rank must be updated too.
This is done because we use a reference to the parent group when we look for the user rank in the group mw.
If this user is not in a parent group -> this wouldn't effect any groups
 */
pub async fn change_rank(group_data: &InternalGroupDataComplete, user_id: impl Into<UserId>, new_rank: i32) -> AppRes<()>
{
	let user_id = user_id.into();

	group_user_model::update_rank(
		&group_data.group_data.id,
		group_data.user_data.rank,
		&user_id,
		new_rank,
	)
	.await?;

	//delete user cache of the changed user
	let key_group = get_group_user_cache_key(&group_data.group_data.app_id, &group_data.group_data.id, &user_id);

	cache::delete(&key_group).await;

	Ok(())
}

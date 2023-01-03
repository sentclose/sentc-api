use std::future::Future;

use sentc_crypto_common::group::{GroupKeysForNewMember, GroupKeysForNewMemberServerInput};
use sentc_crypto_common::{AppId, GroupId, UserId};
use server_core::cache;

use crate::group::group_entities::{GroupInviteReq, InternalGroupDataComplete};
use crate::group::group_model;
use crate::group::group_user::group_user_model;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};
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

pub fn get_invite_req(app_id: AppId, user_id: UserId, last_fetched_time: u128, last_id: GroupId)
	-> impl Future<Output = AppRes<Vec<GroupInviteReq>>>
{
	group_user_model::get_invite_req_to_user(app_id, user_id, last_fetched_time, last_id)
}

/**
# Group invite request to a non group member user

The invited user must accept the invite to join the group
*/
pub async fn invite_request(
	group_data: &InternalGroupDataComplete,
	input: GroupKeysForNewMemberServerInput,
	invited_user: UserId,
	user_type: NewUserType,
) -> AppRes<Option<String>>
{
	if input.keys.is_empty() {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupNoKeys,
			"No group keys for the user".to_string(),
			None,
		));
	}

	if input.keys.len() > 100 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupTooManyKeys,
			"Too many group keys for the user. Split the keys and use pagination".to_string(),
			None,
		));
	}

	if group_data.group_data.invite == 0 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupInviteStop,
			"No invites allowed for this group".to_string(),
			None,
		));
	}

	if let NewUserType::Group = user_type {
		//only connected groups can have other groups as member
		//check in the model if the group to invite a non connected group
		if !group_data.group_data.is_connected_group {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::GroupJoinAsConnectedGroup,
				"Can't invite another group when this group is not a connected group".to_string(),
				None,
			));
		}
	}

	let session_id = group_user_model::invite_request(
		group_data.group_data.id.to_string(),
		invited_user.to_string(),
		input.keys,
		input.key_session,
		group_data.user_data.rank,
		user_type,
	)
	.await?;

	Ok(session_id)
}

/**
# Invite a non group member user and accept the invite

The first half is the same as invite_request but after accept the invite request without new request
*/
pub async fn invite_auto(
	group_data: &InternalGroupDataComplete,
	input: GroupKeysForNewMemberServerInput,
	invited_user: UserId,
	user_type: NewUserType,
) -> AppRes<Option<String>>
{
	let session_id = invite_request(group_data, input, invited_user.to_string(), user_type).await?;

	group_user_model::accept_invite(group_data.group_data.id.to_string(), invited_user).await?;

	Ok(session_id)
}

pub async fn insert_user_keys_via_session(
	group_id: GroupId,
	key_session_id: String,
	insert_type: InsertNewUserType,
	input: Vec<GroupKeysForNewMember>,
) -> AppRes<()>
{
	if input.is_empty() {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupNoKeys,
			"No group keys for the user".to_string(),
			None,
		));
	}

	if input.len() > 100 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupTooManyKeys,
			"Too many group keys for the user. Split the keys and use pagination".to_string(),
			None,
		));
	}

	group_user_model::insert_user_keys_via_session(group_id, key_session_id, input, insert_type).await?;

	Ok(())
}

pub async fn leave_group(group_data: &InternalGroupDataComplete, real_user_id: Option<UserId>) -> AppRes<()>
{
	if let (Some(g_a_m), Some(id)) = (&group_data.user_data.get_values_from_group_as_member, real_user_id) {
		//if user got access by group as member -> check the rank of the user in the real group.
		// this is important because only group admins can leave a connected group

		//do this check everytime with db look up
		// because the rank in the group data only shows the relative rank in the connected group, not the real rank in the group.
		let group_as_member_user_data = group_model::get_internal_group_user_data(g_a_m.clone(), id.clone()).await?;

		match group_as_member_user_data {
			Some(data) => {
				group_model::check_group_rank(data.rank, 1)?;
			},
			None => {
				return Err(HttpErr::new(
					400,
					ApiErrorCodes::GroupUserRank,
					"Wrong group rank for this action".to_string(),
					None,
				));
			},
		}
	}

	group_user_model::user_leave_group(
		group_data.group_data.id.to_string(),
		group_data.user_data.user_id.to_string(),
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

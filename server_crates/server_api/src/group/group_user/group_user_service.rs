use sentc_crypto_common::group::GroupKeysForNewMemberServerInput;
use sentc_crypto_common::{AppId, GroupId, UserId};

use crate::group::group_entities::{GroupInviteReq, InternalGroupDataComplete};
use crate::group::group_user::group_user_model;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

pub async fn get_invite_req(app_id: AppId, user_id: UserId, last_fetched_time: u128, last_id: GroupId) -> AppRes<Vec<GroupInviteReq>>
{
	let reqs = group_user_model::get_invite_req_to_user(app_id, user_id, last_fetched_time, last_id).await?;

	Ok(reqs)
}

/**
# Group invite request to a non group member user

The invited user must accept the invite to join the group
*/
pub async fn invite_request(
	group_data: &InternalGroupDataComplete,
	input: GroupKeysForNewMemberServerInput,
	invited_user: UserId,
) -> AppRes<Option<String>>
{
	if input.keys.len() == 0 {
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

	let session_id = group_user_model::invite_request(
		group_data.group_data.id.to_string(),
		invited_user.to_string(),
		input.keys,
		input.key_session,
		group_data.user_data.rank,
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
) -> AppRes<Option<String>>
{
	let session_id = invite_request(group_data, input, invited_user.to_string()).await?;

	group_user_model::accept_invite(group_data.group_data.id.to_string(), invited_user).await?;

	Ok(session_id)
}
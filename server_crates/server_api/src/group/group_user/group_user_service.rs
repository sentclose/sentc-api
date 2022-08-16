use sentc_crypto_common::{AppId, GroupId, UserId};

use crate::group::group_entities::GroupInviteReq;
use crate::group::group_user::group_user_model;
use crate::util::api_res::AppRes;

pub async fn get_invite_req(app_id: AppId, user_id: UserId, last_fetched_time: u128, last_id: GroupId) -> AppRes<Vec<GroupInviteReq>>
{
	let reqs = group_user_model::get_invite_req_to_user(app_id, user_id, last_fetched_time, last_id).await?;

	Ok(reqs)
}

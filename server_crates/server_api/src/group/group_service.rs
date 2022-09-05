use sentc_crypto_common::group::CreateData;
use sentc_crypto_common::{AppId, GroupId, UserId};

use crate::group::group_model;
use crate::util::api_res::AppRes;

pub async fn create_group(
	app_id: AppId,
	user_id: UserId,
	input: CreateData,
	group_type: i32,
	parent_group_id: Option<GroupId>,
	user_rank: Option<i32>,
) -> AppRes<GroupId>
{
	let group_id = group_model::create(app_id, user_id, input, parent_group_id, user_rank, group_type).await?;

	Ok(group_id)
}

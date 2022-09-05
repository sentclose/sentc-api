use sentc_crypto_common::group::CreateData;
use sentc_crypto_common::{AppId, GroupId, UserId};
use server_core::cache;

use crate::file::file_service;
use crate::group::group_model;
use crate::util::api_res::AppRes;
use crate::util::get_group_cache_key;

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

pub async fn delete_group(app_id: AppId, group_id: GroupId, user_rank: i32) -> AppRes<()>
{
	let children = group_model::delete(app_id.to_string(), group_id.to_string(), user_rank).await?;

	//children incl. the deleted group
	file_service::delete_file_for_group(app_id.as_str(), group_id.as_str(), children).await?;

	let key_group = get_group_cache_key(app_id.as_str(), group_id.as_str());
	cache::delete(key_group.as_str()).await;

	Ok(())
}

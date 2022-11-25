use std::future::Future;

use sentc_crypto_common::group::CreateData;
use sentc_crypto_common::{AppId, GroupId, SymKeyId, UserId};
use server_core::cache;

use crate::file::file_service;
use crate::group::group_entities::GroupUserKeys;
use crate::group::group_model;
use crate::util::api_res::AppRes;
use crate::util::get_group_cache_key;

pub fn create_group(
	app_id: AppId,
	user_id: UserId,
	input: CreateData,
	group_type: i32,
	parent_group_id: Option<GroupId>,
	user_rank: Option<i32>,
	connected_group: Option<GroupId>,
) -> impl Future<Output = AppRes<GroupId>>
{
	group_model::create(
		app_id,
		user_id,
		input,
		parent_group_id,
		user_rank,
		group_type,
		connected_group,
	)
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

pub fn delete_user_group(app_id: AppId, group_id: GroupId) -> impl Future<Output = AppRes<()>>
{
	group_model::delete_user_group(app_id, group_id)
}

pub fn get_user_group_keys(
	app_id: AppId,
	group_id: GroupId,
	user_id: UserId,
	last_fetched_time: u128,
	last_k_id: SymKeyId,
) -> impl Future<Output = AppRes<Vec<GroupUserKeys>>>
{
	group_model::get_user_group_keys(app_id, group_id, user_id, last_fetched_time, last_k_id)
}

/**
# Get a single key
*/
pub fn get_user_group_key(app_id: AppId, group_id: GroupId, user_id: UserId, key_id: SymKeyId) -> impl Future<Output = AppRes<GroupUserKeys>>
{
	group_model::get_user_group_key(app_id, group_id, user_id, key_id)
}

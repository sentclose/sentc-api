use std::future::Future;

use sentc_crypto_common::group::{CreateData, GroupLightServerData, GroupUserAccessBy};
use sentc_crypto_common::{AppId, GroupId, SymKeyId, UserId};
use server_core::cache;

use crate::file::file_service;
use crate::group::group_entities::{GroupChildrenList, GroupServerData, GroupUserKeys, InternalGroupDataComplete};
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
	is_connected_group: bool,
) -> impl Future<Output = AppRes<(GroupId, SymKeyId)>>
{
	group_model::create(
		app_id,
		user_id,
		input,
		parent_group_id,
		user_rank,
		group_type,
		connected_group,
		is_connected_group,
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

pub fn get_user_group_light_data(group_data: &InternalGroupDataComplete) -> GroupLightServerData
{
	let (parent, access_by) = extract_parent_and_access_by(group_data);

	GroupLightServerData {
		group_id: group_data.group_data.id.to_string(),
		parent_group_id: parent,
		rank: group_data.user_data.rank,
		created_time: group_data.group_data.time,
		joined_time: group_data.user_data.joined_time,
		access_by,
	}
}

pub async fn get_user_group_data(group_data: &InternalGroupDataComplete) -> AppRes<GroupServerData>
{
	let app_id = &group_data.group_data.app_id;
	let group_id = &group_data.group_data.id;
	let user_id = &group_data.user_data.user_id;

	let keys = get_user_group_keys(
		app_id.to_string(),
		group_id.to_string(),
		user_id.to_string(),
		0, //fetch the first page
		"".to_string(),
	)
	.await?;

	let key_update = group_model::check_for_key_update(app_id.to_string(), user_id.to_string(), group_id.to_string()).await?;

	let (parent, access_by) = extract_parent_and_access_by(group_data);

	Ok(GroupServerData {
		group_id: group_id.to_string(),
		parent_group_id: parent,
		keys,
		key_update,
		rank: group_data.user_data.rank,
		created_time: group_data.group_data.time,
		joined_time: group_data.user_data.joined_time,
		access_by,
		is_connected_group: group_data.group_data.is_connected_group,
	})
}

fn extract_parent_and_access_by(group_data: &InternalGroupDataComplete) -> (Option<String>, GroupUserAccessBy)
{
	let parent = match &group_data.group_data.parent {
		Some(p) => Some(p.to_string()),
		None => None,
	};

	//tell the frontend how thi group as access
	let access_by = match (
		&group_data.user_data.get_values_from_group_as_member,
		&group_data.user_data.get_values_from_parent,
	) {
		//the user is in a group which is member in a parent group
		(Some(v_as_member), Some(v_as_parent)) => {
			GroupUserAccessBy::GroupAsUserAsParent {
				group_as_user: v_as_member.to_string(),
				parent: v_as_parent.to_string(),
			}
		},
		(Some(v_as_member), None) => GroupUserAccessBy::GroupAsUser(v_as_member.to_string()),
		(None, Some(v_as_parent)) => GroupUserAccessBy::Parent(v_as_parent.to_string()),
		(None, None) => GroupUserAccessBy::User,
	};

	(parent, access_by)
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

pub fn get_first_level_children(
	app_id: AppId,
	group_id: GroupId,
	last_fetched_time: u128,
	last_id: GroupId,
) -> impl Future<Output = AppRes<Vec<GroupChildrenList>>>
{
	group_model::get_first_level_children(app_id, group_id, last_fetched_time, last_id)
}

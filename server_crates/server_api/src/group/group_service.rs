use std::future::Future;

use rustgram_server_util::cache;
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::group::{CreateData, GroupLightServerData, GroupUserAccessBy};
use sentc_crypto_common::{AppId, GroupId, SymKeyId, UserId};
use server_api_common::group::group_entities::InternalGroupDataComplete;
use server_api_common::util::get_group_cache_key;

use crate::group::group_entities::{GroupChildrenList, GroupServerData, GroupUserKeys};
use crate::group::group_model;
use crate::sentc_group_entities::{GroupHmacData, GroupSortableData};
use crate::sentc_user_entities::UserPublicKeyDataEntity;

pub fn create_group_light<'a>(
	app_id: impl Into<AppId> + 'a,
	user_id: impl Into<UserId> + 'a,
	group_type: i32,
	parent_group_id: Option<GroupId>,
	user_rank: Option<i32>,
	connected_group: Option<GroupId>,
	is_connected_group: bool,
) -> impl Future<Output = AppRes<GroupId>> + 'a
{
	group_model::create_light(
		app_id,
		user_id,
		parent_group_id,
		user_rank,
		group_type,
		connected_group,
		is_connected_group,
	)
}

pub fn create_group<'a>(
	app_id: impl Into<AppId> + 'a,
	user_id: impl Into<UserId> + 'a,
	input: CreateData,
	group_type: i32,
	parent_group_id: Option<GroupId>,
	user_rank: Option<i32>,
	connected_group: Option<GroupId>,
	is_connected_group: bool,
) -> impl Future<Output = AppRes<(GroupId, SymKeyId)>> + 'a
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

pub async fn delete_group(app_id: &str, group_id: &str, user_rank: i32) -> AppRes<()>
{
	let children = group_model::delete(app_id, group_id, user_rank).await?;

	//children incl. the deleted group
	server_api_common::file::delete_file_for_group(app_id, group_id, children).await?;

	let key_group = get_group_cache_key(app_id, group_id);
	cache::delete(key_group.as_str()).await?;

	Ok(())
}

pub fn delete_user_group<'a>(app_id: impl Into<AppId> + 'a, group_id: impl Into<GroupId> + 'a) -> impl Future<Output = AppRes<()>> + 'a
{
	group_model::delete_user_group(app_id, group_id)
}

pub async fn stop_invite(app_id: &str, group_id: &str, user_rank: i32) -> AppRes<()>
{
	group_model::stop_invite(app_id, group_id, user_rank).await?;

	let key_group = get_group_cache_key(app_id, group_id);
	cache::delete(key_group.as_str()).await?;

	Ok(())
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
		is_connected_group: group_data.group_data.is_connected_group,
	}
}

pub async fn get_user_group_data(group_data: &InternalGroupDataComplete) -> AppRes<GroupServerData>
{
	let app_id = &group_data.group_data.app_id;
	let group_id = &group_data.group_data.id;
	let user_id = &group_data.user_data.user_id;

	let (keys, hmac_keys, sortable_keys, key_update) = tokio::try_join!(
		get_user_group_keys(app_id, group_id, user_id, 0, "",),
		get_group_hmac(app_id, group_id, 0, "",),
		get_group_sortable(app_id, group_id, 0, ""),
		group_model::check_for_key_update(app_id, user_id, group_id)
	)?;

	let (parent, access_by) = extract_parent_and_access_by(group_data);

	Ok(GroupServerData {
		group_id: group_id.to_string(),
		parent_group_id: parent,
		keys,
		hmac_keys,
		sortable_keys,
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

pub fn get_group_hmac<'a>(
	app_id: impl Into<AppId> + 'a,
	group_id: impl Into<GroupId> + 'a,
	last_fetched_time: u128,
	last_k_id: impl Into<SymKeyId> + 'a,
) -> impl Future<Output = AppRes<Vec<GroupHmacData>>> + 'a
{
	group_model::get_group_hmac(app_id, group_id, last_fetched_time, last_k_id)
}

pub fn get_group_sortable<'a>(
	app_id: impl Into<AppId> + 'a,
	group_id: impl Into<GroupId> + 'a,
	last_fetched_time: u128,
	last_k_id: impl Into<SymKeyId> + 'a,
) -> impl Future<Output = AppRes<Vec<GroupSortableData>>> + 'a
{
	group_model::get_group_sortable(app_id, group_id, last_fetched_time, last_k_id)
}

pub fn get_user_group_keys<'a>(
	app_id: impl Into<AppId> + 'a,
	group_id: impl Into<GroupId> + 'a,
	user_id: impl Into<UserId> + 'a,
	last_fetched_time: u128,
	last_k_id: impl Into<SymKeyId> + 'a,
) -> impl Future<Output = AppRes<Vec<GroupUserKeys>>> + 'a
{
	group_model::get_user_group_keys(app_id, group_id, user_id, last_fetched_time, last_k_id)
}

pub fn get_public_key_data<'a>(
	app_id: impl Into<AppId> + 'a,
	group_id: impl Into<GroupId> + 'a,
) -> impl Future<Output = AppRes<UserPublicKeyDataEntity>> + 'a
{
	group_model::get_public_key_data(app_id, group_id)
}

/**
# Get a single key
*/
pub fn get_user_group_key<'a>(
	app_id: impl Into<AppId> + 'a,
	group_id: impl Into<GroupId> + 'a,
	user_id: impl Into<UserId> + 'a,
	key_id: impl Into<SymKeyId> + 'a,
) -> impl Future<Output = AppRes<GroupUserKeys>> + 'a
{
	group_model::get_user_group_key(app_id, group_id, user_id, key_id)
}

pub fn get_first_level_children<'a>(
	app_id: impl Into<AppId> + 'a,
	group_id: impl Into<GroupId> + 'a,
	last_fetched_time: u128,
	last_id: impl Into<GroupId> + 'a,
) -> impl Future<Output = AppRes<Vec<GroupChildrenList>>> + 'a
{
	group_model::get_first_level_children(app_id, group_id, last_fetched_time, last_id)
}

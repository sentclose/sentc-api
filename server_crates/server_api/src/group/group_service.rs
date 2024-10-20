use rustgram_server_util::cache;
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::group::{GroupLightServerData, GroupUserAccessBy};
use server_api_common::group::group_entities::InternalGroupDataComplete;
use server_api_common::util::get_group_cache_key;

pub use self::group_model::{
	create as create_group,
	create_light as create_group_light,
	delete_user_group,
	get_first_level_children,
	get_group_hmac,
	get_group_sortable,
	get_public_key_data,
	get_user_group_key,
	get_user_group_keys,
};
use crate::group::group_entities::GroupServerData;
use crate::group::group_model;

pub async fn delete_group(app_id: &str, group_id: &str, user_rank: i32) -> AppRes<()>
{
	let children = group_model::delete(app_id, group_id, user_rank).await?;

	//children incl. the deleted group
	server_api_common::file::delete_file_for_group(app_id, group_id, children).await?;

	let key_group = get_group_cache_key(app_id, group_id);
	cache::delete(key_group.as_str()).await?;

	Ok(())
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

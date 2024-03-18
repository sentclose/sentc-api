use rustgram_server_util::db::query_first;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::set_params;
use sentc_crypto_common::{AppId, GroupId, UserId};

use crate::group::group_entities::{InternalGroupData, InternalUserGroupData, InternalUserGroupDataFromParent};
use crate::group::GROUP_TYPE_NORMAL;
use crate::ApiErrorCodes;

pub(crate) async fn get_internal_group_data(app_id: impl Into<AppId>, group_id: impl Into<GroupId>) -> AppRes<InternalGroupData>
{
	//language=SQL
	let sql = "SELECT id as group_id, app_id, parent, time, invite, is_connected_group FROM sentc_group WHERE app_id = ? AND id = ? AND type = ?";
	query_first(sql, set_params!(app_id.into(), group_id.into(), GROUP_TYPE_NORMAL))
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(400, ApiErrorCodes::GroupAccess, "No access to this group"))
}

pub(crate) async fn get_user_from_parent_groups(
	group_id: impl Into<GroupId>,
	user_id: impl Into<UserId>,
) -> AppRes<Option<InternalUserGroupDataFromParent>>
{
	//search via recursion all parent ids for this group.
	//https://www.mysqltutorial.org/mysql-adjacency-list-tree/
	//https://rolandgeng.de/managing-trees-in-mysql-using-the-adjacency-list-model/
	/*
		//language=SQL
		let sql = r"
	WITH RECURSIVE parents (id, parent) AS (
		SELECT id, parent FROM sentc_group WHERE id = ?

		UNION ALL

		SELECT g.id, g.parent FROM parents p
				  JOIN sentc_group g ON p.parent = g.id
	)
	SELECT id FROM parents
	";
	*/

	//language=SQL
	let sql = r"
SELECT group_id, time, `rank` FROM sentc_group_user WHERE user_id = ? AND group_id IN (
    WITH RECURSIVE parents (id, parent) AS ( 
		SELECT id, parent FROM sentc_group WHERE id = ?
										   
		UNION ALL 
		
		SELECT g.id, g.parent FROM parents p 
				  JOIN sentc_group g ON p.parent = g.id
	)
	SELECT id FROM parents
) LIMIT 1
";

	query_first(sql, set_params!(user_id.into(), group_id.into())).await
}

pub(crate) async fn get_internal_group_user_data(group_id: impl Into<GroupId>, user_id: impl Into<UserId>) -> AppRes<Option<InternalUserGroupData>>
{
	//language=SQL
	let sql = "SELECT user_id, time, `rank` FROM sentc_group_user WHERE group_id = ? AND user_id = ?";
	query_first(sql, set_params!(group_id.into(), user_id.into())).await
}

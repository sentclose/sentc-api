use sentc_crypto_common::group::CreateData;
use sentc_crypto_common::{AppId, GroupId, SymKeyId, UserId};
use server_core::db::{exec, exec_string, exec_transaction, get_in, query, query_first, query_string, I32Entity, StringEntity, TransactionData};
use server_core::{get_time, set_params, set_params_vec};
use uuid::Uuid;

use crate::group::group_entities::{
	GroupChildrenList,
	GroupUserKeys,
	InternalGroupData,
	InternalUserGroupData,
	InternalUserGroupDataFromParent,
	ListGroups,
};
use crate::group::{GROUP_TYPE_NORMAL, GROUP_TYPE_USER};
use crate::user::user_entities::UserPublicKeyDataEntity;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

pub(crate) async fn get_internal_group_data(app_id: AppId, group_id: GroupId) -> AppRes<InternalGroupData>
{
	//language=SQL
	let sql = "SELECT id as group_id, app_id, parent, time, invite, is_connected_group FROM sentc_group WHERE app_id = ? AND id = ? AND type = ?";
	let group: Option<InternalGroupData> = query_first(sql, set_params!(app_id, group_id, GROUP_TYPE_NORMAL)).await?;

	match group {
		Some(d) => Ok(d),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::GroupAccess,
				"No access to this group".to_string(),
				None,
			))
		},
	}
}

pub(crate) async fn get_user_from_parent_groups(group_id: GroupId, user_id: UserId) -> AppRes<Option<InternalUserGroupDataFromParent>>
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

	let group_data: Option<InternalUserGroupDataFromParent> = query_first(sql, set_params!(user_id, group_id)).await?;

	Ok(group_data)
}

pub(crate) async fn get_internal_group_user_data(group_id: GroupId, user_id: UserId) -> AppRes<Option<InternalUserGroupData>>
{
	//language=SQL
	let sql = "SELECT user_id, time, `rank` FROM sentc_group_user WHERE group_id = ? AND user_id = ?";
	let group_data: Option<InternalUserGroupData> = query_first(sql, set_params!(group_id, user_id)).await?;

	Ok(group_data)
}

//__________________________________________________________________________________________________

/**
Get every other group keys with pagination.

This keys are normally cached in the client, so it should be fetched once for each client.

New keys from key update are fetched by the key update fn

For child group: use the parent group id as user id.
*/
pub(super) async fn get_user_group_keys(
	app_id: AppId,
	group_id: GroupId,
	user_id: UserId,
	last_fetched_time: u128,
	last_k_id: SymKeyId,
) -> AppRes<Vec<GroupUserKeys>>
{
	//language=SQL
	let sql = r"
SELECT 
    k_id,
    encrypted_group_key, 
    group_key_alg, 
    encrypted_private_key,
    public_key,
    private_key_pair_alg,
    uk.encrypted_group_key_key_id,
    uk.time,
    encrypted_sign_key,
    verify_key,
    keypair_sign_alg
FROM 
    sentc_group_keys k, 
    sentc_group_user_keys uk
WHERE 
    user_id = ? AND 
    k.group_id = ? AND 
    k.id = k_id AND 
    app_id = ?"
		.to_string();

	let (sql1, params) = if last_fetched_time > 0 {
		//there is a last fetched time time -> set the last fetched time to the params list
		let sql = sql + " AND uk.time <= ? AND (uk.time < ? OR (uk.time = ? AND k_id > ?)) ORDER BY uk.time DESC, k_id LIMIT 50";

		(
			sql,
			set_params!(
				user_id,
				group_id,
				app_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_k_id
			),
		)
	} else {
		let sql = sql + " ORDER BY uk.time DESC, k_id LIMIT 50";
		(sql, set_params!(user_id, group_id, app_id))
	};

	let user_keys: Vec<GroupUserKeys> = query_string(sql1, params).await?;

	Ok(user_keys)
}

pub(super) async fn get_user_group_key(app_id: AppId, group_id: GroupId, user_id: UserId, key_id: SymKeyId) -> AppRes<GroupUserKeys>
{
	//language=SQL
	let sql = r"
SELECT 
    k_id,
    encrypted_group_key, 
    group_key_alg, 
    encrypted_private_key,
    public_key,
    private_key_pair_alg,
    uk.encrypted_group_key_key_id,
    uk.time,
    encrypted_sign_key,
    verify_key,
    keypair_sign_alg
FROM 
    sentc_group_keys k, 
    sentc_group_user_keys uk
WHERE 
    user_id = ? AND 
    k.group_id = ? AND 
    k_id = ? AND
    k.id = k_id AND 
    app_id = ?";

	let key: Option<GroupUserKeys> = query_first(sql, set_params!(user_id, group_id, key_id, app_id)).await?;

	match key {
		Some(k) => Ok(k),
		None => {
			Err(HttpErr::new(
				200,
				ApiErrorCodes::GroupKeyNotFound,
				"Group key not found".to_string(),
				None,
			))
		},
	}
}

/**
Get the info if there was a key update in the mean time

For child group: use the parent group id as user id.
*/
pub(super) async fn check_for_key_update(app_id: AppId, user_id: UserId, group_id: GroupId) -> AppRes<bool>
{
	//check for key update
	//language=SQL
	let sql = r"
SELECT 1 
FROM 
    sentc_group_user_key_rotation gkr,
    sentc_group g
WHERE
    user_id = ? AND
    app_id = ? AND 
    g.id = gkr.group_id AND
    g.id = ?
LIMIT 1";

	let key_update: Option<I32Entity> = query_first(sql, set_params!(user_id, app_id, group_id)).await?;

	match key_update {
		Some(_) => Ok(true),
		None => Ok(false),
	}
}

pub(super) async fn create(
	app_id: AppId,
	user_id: UserId,
	data: CreateData,
	parent_group_id: Option<GroupId>,
	user_rank: Option<i32>,
	group_type: i32,
	connected_group: Option<GroupId>,
	is_connected_group: bool,
) -> AppRes<(GroupId, SymKeyId)>
{
	let (insert_user_id, user_type, group_connected) = match (&parent_group_id, user_rank, connected_group) {
		(Some(p), Some(r), None) => {
			//test here if the user has access to create a child group in this group
			check_group_rank(r, 1)?;

			//when it is a parent group -> use this id as user id for the group user insert
			//when the parent group is a connected group, so the children are too
			(p.to_string(), 1, is_connected_group)
		},
		(None, Some(r), Some(c)) => {
			check_group_rank(r, 1)?;

			// user type is group as member
			(c, 2, true)
		},
		//when parent group is some then user rank must be some too,
		// because this is set by the controller and not the user.
		_ => (user_id, 0, false),
	};

	let group_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//language=SQL
	let sql_group = "INSERT INTO sentc_group (id, app_id, parent, identifier, time, type, is_connected_group, invite) VALUES (?,?,?,?,?,?,?,?)";
	let group_params = set_params!(
		group_id.to_string(),
		app_id.to_string(),
		parent_group_id,
		"".to_string(),
		time.to_string(),
		group_type,
		group_connected,
		1
	);

	let group_key_id = Uuid::new_v4().to_string();

	//language=SQL
	let sql_group_data = r"
INSERT INTO sentc_group_keys 
    (
     id, 
     group_id, 
     app_id,
     private_key_pair_alg, 
     encrypted_private_key, 
     public_key, 
     group_key_alg, 
     encrypted_ephemeral_key, 
     encrypted_group_key_by_eph_key,
     previous_group_key_id,
     time,
     encrypted_sign_key,
     verify_key,
     keypair_sign_alg
     ) 
VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

	let encrypted_ephemeral_key: Option<String> = None;
	let encrypted_group_key_by_eph_key: Option<String> = None;
	let previous_group_key_id: Option<String> = None;

	let group_data_params = set_params!(
		group_key_id.to_string(),
		group_id.to_string(),
		app_id.to_string(),
		data.keypair_encrypt_alg,
		data.encrypted_private_group_key,
		data.public_group_key,
		data.group_key_alg,
		encrypted_ephemeral_key,
		encrypted_group_key_by_eph_key,
		previous_group_key_id,
		time.to_string(),
		data.encrypted_sign_key,
		data.verify_key,
		data.keypair_sign_alg
	);

	//insert he creator => rank = 0
	//handle parent group as the creator.

	//language=SQL
	let sql_group_user = "INSERT INTO sentc_group_user (user_id, group_id, time, `rank`, type) VALUES (?,?,?,?,?)";
	let group_user_params = set_params!(
		insert_user_id.to_string(),
		group_id.to_string(),
		time.to_string(),
		0,
		user_type
	);

	//language=SQL
	let sql_group_user_keys = r"
INSERT INTO sentc_group_user_keys 
    (
     k_id, 
     user_id, 
     group_id, 
     encrypted_group_key, 
     encrypted_alg, 
     encrypted_group_key_key_id,
     time
     ) 
VALUES (?,?,?,?,?,?,?)";

	let group_user_keys_params = set_params!(
		group_key_id.clone(),
		insert_user_id,
		group_id.to_string(),
		data.encrypted_group_key,
		data.encrypted_group_key_alg,
		data.creator_public_key_id,
		time.to_string()
	);

	exec_transaction(vec![
		TransactionData {
			sql: sql_group,
			params: group_params,
		},
		TransactionData {
			sql: sql_group_data,
			params: group_data_params,
		},
		TransactionData {
			sql: sql_group_user,
			params: group_user_params,
		},
		TransactionData {
			sql: sql_group_user_keys,
			params: group_user_keys_params,
		},
	])
	.await?;

	Ok((group_id, group_key_id))
}

pub(super) async fn delete_user_group(app_id: AppId, group_id: GroupId) -> AppRes<()>
{
	//don't delete children because user group won't have children

	//language=SQL
	let sql = r"
DELETE 
FROM sentc_group 
WHERE 
    app_id = ? AND 
    type = ? AND 
    id = ?";

	exec(sql, set_params!(app_id, GROUP_TYPE_USER, group_id)).await?;

	Ok(())
}

pub(super) async fn delete(app_id: AppId, group_id: GroupId, user_rank: i32) -> AppRes<Vec<String>>
{
	//check with app id to make sure the user is in the right group
	check_group_rank(user_rank, 1)?;

	//language=SQL
	let sql = "DELETE FROM sentc_group WHERE id = ? AND app_id = ? AND type = ?";
	exec(
		sql,
		set_params!(group_id.to_string(), app_id.to_string(), GROUP_TYPE_NORMAL),
	)
	.await?;

	//delete the children via recursion, can't delete them directly because sqlite don't support delete from multiple tables
	//can't delete it via trigger because it is the same table
	//can't delete it via on delete cascade because the trigger for the children won't run, so we are left with garbage data.
	let children = get_children_to_parent(group_id.to_string(), app_id.to_string()).await?;

	let mut children_out = Vec::with_capacity(children.len());

	if !children.is_empty() {
		for child in &children {
			children_out.push(child.0.to_string());
		}

		let get_in = get_in(&children);

		//language=SQLx
		let sql_delete_child = format!("DELETE FROM sentc_group WHERE id IN ({})", get_in);

		//set params with vec
		exec_string(sql_delete_child, set_params_vec!(children)).await?;
	}

	//delete the rest of the user group keys, this is the rest from user invite but this wont get deleted when group user gets deleted
	//important: do this after the delete!

	//language=SQL
	let sql = "DELETE FROM sentc_group_user_keys WHERE group_id = ?";

	exec(sql, set_params!(group_id)).await?;

	Ok(children_out)
}

pub(super) async fn stop_invite(app_id: AppId, group_id: GroupId, user_rank: i32) -> AppRes<()>
{
	check_group_rank(user_rank, 1)?;

	#[cfg(feature = "mysql")]
	//language=SQL
	let sql = "UPDATE sentc_group SET invite = IF(invite LIKE 0, 1,0) WHERE app_id = ? AND id = ?";

	#[cfg(feature = "sqlite")]
	let sql = "UPDATE sentc_group SET invite = IIF(invite LIKE 0, 1,0) WHERE app_id = ? AND id = ?";

	exec(sql, set_params!(app_id, group_id)).await?;

	Ok(())
}

/**
user here the cache from the mw

Then check if the rank fits
*/
pub(super) fn check_group_rank(user_rank: i32, req_rank: i32) -> AppRes<()>
{
	if user_rank > req_rank {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupUserRank,
			"Wrong group rank for this action".to_string(),
			None,
		));
	}

	Ok(())
}

pub(super) async fn get_children_to_parent(group_id: GroupId, app_id: AppId) -> AppRes<Vec<StringEntity>>
{
	//language=SQL
	let sql = r"
WITH RECURSIVE children (id) AS ( 
    SELECT g.id from sentc_group g WHERE g.parent = ? AND g.app_id = ?
                                   
    UNION ALL 
        
    SELECT g1.id FROM children c
            JOIN sentc_group g1 ON c.id = g1.parent AND g1.app_id = ?
)
SELECT * FROM children
";

	let children: Vec<StringEntity> = query(sql, set_params!(group_id, app_id.to_string(), app_id)).await?;

	Ok(children)
}

//__________________________________________________________________________________________________

pub(super) async fn get_all_groups_to_user(app_id: AppId, user_id: UserId, last_fetched_time: u128, last_id: GroupId) -> AppRes<Vec<ListGroups>>
{
	//language=SQL
	let sql = r"
SELECT id, g.time as time, gu.time as joined_time, `rank`, parent
FROM sentc_group g, sentc_group_user gu
WHERE 
    app_id = ? AND 
    group_id = id AND 
    user_id = ? AND 
    (gu.type = 0 OR gu.type = 2)"
		.to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND g.time >= ? AND (g.time > ? OR (g.time = ? AND id > ?)) ORDER BY time, group_id LIMIT 50";
		(
			sql,
			set_params!(
				app_id,
				user_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id
			),
		)
	} else {
		let sql = sql + " ORDER BY time, group_id LIMIT 50";

		(sql, set_params!(app_id, user_id))
	};

	let list: Vec<ListGroups> = query_string(sql, params).await?;

	Ok(list)
}

pub(super) async fn get_public_key_data(app_id: AppId, group_id: GroupId) -> AppRes<UserPublicKeyDataEntity>
{
	//language=SQL
	let sql = r"
SELECT id, public_key, private_key_pair_alg 
FROM sentc_group_keys 
WHERE 
    app_id = ? AND 
    group_id = ? 
ORDER BY time DESC 
LIMIT 1";

	//the same as user keys, because in this case the group is like a user
	let data: Option<UserPublicKeyDataEntity> = query_first(sql, set_params!(app_id, group_id)).await?;

	match data {
		Some(d) => Ok(d),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::GroupKeyNotFound,
				"Public key from this group was not found".to_string(),
				None,
			))
		},
	}
}

pub(super) async fn get_first_level_children(
	app_id: AppId,
	group_id: GroupId,
	last_fetched_time: u128,
	last_id: GroupId,
) -> AppRes<Vec<GroupChildrenList>>
{
	//get the direct children, where parent id == group id
	//last id is the last child id

	//language=SQL
	let sql = "SELECT id, time, parent FROM sentc_group WHERE parent = ? AND app_id = ?".to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND time >= ? AND (time > ? OR (time = ? AND id > ?)) ORDER BY time, id LIMIT 50";
		(
			sql,
			set_params!(
				group_id,
				app_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id
			),
		)
	} else {
		let sql = sql + " ORDER BY time, id LIMIT 50";
		(sql, set_params!(group_id, app_id))
	};

	let list: Vec<GroupChildrenList> = query_string(sql, params).await?;

	Ok(list)
}

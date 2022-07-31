use sentc_crypto_common::group::CreateData;
use sentc_crypto_common::{AppId, GroupId, UserId};
use uuid::Uuid;

use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::core::db::{exec, exec_transaction, query, query_first, TransactionData};
use crate::core::get_time;
use crate::group::group_entities::{
	GroupKeyUpdate,
	GroupKeyUpdateReady,
	GroupUserData,
	GroupUserKeys,
	InternalGroupData,
	InternalUserGroupData,
	UserGroupRankCheck,
};
use crate::set_params;

pub(crate) async fn get_internal_group_data(app_id: AppId, group_id: GroupId) -> AppRes<InternalGroupData>
{
	//language=SQL
	let sql = "SELECT id as group_id, app_id, parent, time FROM sentc_group WHERE app_id = ? AND id = ?";
	let group: Option<InternalGroupData> = query_first(sql, set_params!(app_id, group_id)).await?;

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

pub(crate) async fn get_internal_group_user_data(group_id: GroupId, user_id: UserId) -> AppRes<InternalUserGroupData>
{
	//language=SQL
	let sql = "SELECT user_id, time, `rank` FROM sentc_group_user WHERE group_id = ? AND user_id = ?";
	let group_data: Option<InternalUserGroupData> = query_first(sql, set_params!(group_id, user_id)).await?;

	match group_data {
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

/**
Get the general group data for init the group in the client.

This info only needs to fetched once for each client, because it is normally cached int he client.
*/
pub(super) async fn get_user_group_data(app_id: AppId, user_id: UserId, group_id: GroupId) -> AppRes<(GroupUserData, Vec<GroupUserKeys>)>
{
	//language=SQL
	let sql = r"
SELECT id, parent, `rank`, g.time as created_time, gu.time as joined_time
FROM 
    sentc_group g,
    sentc_group_user gu
WHERE 
    app_id = ? AND 
    id = ? AND
    user_id = ? AND
    group_id = id";

	let user_group_data: Option<GroupUserData> = query_first(sql, set_params!(app_id, group_id.to_string(), user_id.to_string())).await?;

	let user_group_data = match user_group_data {
		Some(d) => d,
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::GroupUserNotFound,
				"Group user not exists in this group".to_string(),
				None,
			))
		},
	};

	//just a simple query, without time checking and pagination (this is done in other fn)
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
    uk.time
FROM 
    sentc_group_keys k, 
    sentc_group_user_keys uk 
WHERE 
    user_id = ? AND 
    k.group_id = ? AND 
    id = k_id
ORDER BY uk.time DESC LIMIT 50";

	let user_keys: Vec<GroupUserKeys> = query(sql, set_params!(user_id, group_id)).await?;

	Ok((user_group_data, user_keys))
}

/**
Get every other group keys with pagination.

This keys are normally cached in the client, so it should be fetched once for each client.

New keys from key update are fetched by the key update fn
*/
pub(super) async fn get_user_group_keys(app_id: AppId, group_id: GroupId, user_id: UserId, last_fetched_time: u128) -> AppRes<Vec<GroupUserKeys>>
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
    uk.time
FROM 
    sentc_group_keys k, 
    sentc_group_user_keys uk, 
    sentc_group g -- group here for the app id
WHERE 
    user_id = ? AND 
    k.group_id = ? AND 
    k.id = k_id AND 
    g.id = k.group_id AND 
    app_id = ? AND 
    uk.time >= ?
ORDER BY uk.time DESC LIMIT 50";

	let user_keys: Vec<GroupUserKeys> = query(
		sql,
		set_params!(user_id, group_id, app_id, last_fetched_time.to_string()),
	)
	.await?;

	Ok(user_keys)
}

/**
Get the info if there was a key update in the mean time
*/
pub(super) async fn check_for_key_update(app_id: AppId, user_id: UserId, group_id: GroupId) -> AppRes<bool>
{
	//check for key update
	//language=SQL
	let sql = r"
SELECT 1 
FROM 
    sentc_group_keys gk, 
    sentc_group_user_key_rotation gkr,
    sentc_group g
WHERE
    user_id = ? AND
    app_id = ? AND 
    key_id = gk.id AND
    g.id = gk.group_id AND
    g.id = ?
ORDER BY gk.time DESC LIMIT 1";

	let key_update: Option<GroupKeyUpdateReady> = query_first(sql, set_params!(user_id, app_id, group_id)).await?;

	match key_update {
		Some(_) => Ok(true),
		None => Ok(false),
	}
}

pub(super) async fn get_keys_for_key_update(app_id: AppId, group_id: GroupId, user_id: UserId) -> AppRes<Vec<GroupKeyUpdate>>
{
	//check if there was a key rotation, fetch all rotation keys in the table
	//language=SQL
	let sql = r"
SELECT 
    gkr.encrypted_ephemeral_key, 
    gkr.encrypted_eph_key_key_id,	-- the key id of the public key which was used to encrypt the eph key on the server
    encrypted_group_key_by_eph_key,
    previous_group_key_id,
    gk.time
FROM 
    sentc_group_keys gk, 
    sentc_group_user_key_rotation gkr,
    sentc_group g
WHERE user_id = ? AND 
      g.id = ? AND 
      app_id = ? AND 
      key_id = gk.id AND 
      gk.group_id = g.id 
ORDER BY gk.time";

	let out: Vec<GroupKeyUpdate> = query(sql, set_params!(user_id, group_id, app_id)).await?;

	Ok(out)
}

pub(super) async fn create(app_id: AppId, user_id: UserId, data: CreateData) -> AppRes<GroupId>
{
	match &data.parent_group_id {
		None => {},
		Some(p) => {
			//test here if the user has access to create a child group in this group
			check_group_rank_by_fetch(app_id.to_string(), p.to_string(), user_id.to_string(), 1).await?;
		},
	}

	let group_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//language=SQL
	let sql_group = "INSERT INTO sentc_group (id, app_id, parent, identifier, time) VALUES (?,?,?,?,?)";
	let group_params = set_params!(
		group_id.to_string(),
		app_id,
		data.parent_group_id,
		"".to_string(),
		time.to_string()
	);

	let group_key_id = Uuid::new_v4().to_string();

	//language=SQL
	let sql_group_data = r"
INSERT INTO sentc_group_keys 
    (
     id, 
     group_id, 
     private_key_pair_alg, 
     encrypted_private_key, 
     public_key, 
     group_key_alg, 
     encrypted_ephemeral_key, 
     encrypted_group_key_by_eph_key,
     previous_group_key_id,
     time
     ) 
VALUES (?,?,?,?,?,?,?,?,?,?)";

	let encrypted_ephemeral_key: Option<String> = None;
	let encrypted_group_key_by_eph_key: Option<String> = None;
	let previous_group_key_id: Option<String> = None;

	let group_data_params = set_params!(
		group_key_id.to_string(),
		group_id.to_string(),
		data.keypair_encrypt_alg,
		data.encrypted_private_group_key,
		data.public_group_key,
		data.group_key_alg,
		encrypted_ephemeral_key,
		encrypted_group_key_by_eph_key,
		previous_group_key_id,
		time.to_string()
	);

	//insert he creator => rank = 0
	//language=SQL
	let sql_group_user = "INSERT INTO sentc_group_user (user_id, group_id, time, `rank`) VALUES (?,?,?,?)";
	let group_user_params = set_params!(user_id.to_string(), group_id.to_string(), time.to_string(), 0);

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
		group_key_id,
		user_id,
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

	Ok(group_id)
}

pub(super) async fn delete(app_id: AppId, group_id: GroupId, user_rank: i32) -> AppRes<()>
{
	//check with app id to make sure the user is in the right group
	check_group_rank(user_rank, 1)?;

	//language=SQL
	let sql = "DELETE FROM sentc_group WHERE id = ? AND app_id = ?";
	let delete_params = set_params!(group_id.to_string(), app_id.to_string());

	//delete the children
	//language=SQL
	let sql_delete_child = "DELETE FROM sentc_group WHERE parent = ? AND app_id = ?";
	let delete_children_params = set_params!(group_id.to_string(), app_id);

	exec_transaction(vec![
		TransactionData {
			sql,
			params: delete_params,
		},
		TransactionData {
			sql: sql_delete_child,
			params: delete_children_params,
		},
	])
	.await?;

	//delete the rest of the user group keys, this is the rest from user invite but this wont get deleted when group user gets deleted
	//important: do this after the delete!

	//language=SQL
	let sql = "DELETE FROM sentc_group_user_keys WHERE group_id = ?";

	exec(sql, set_params!(group_id)).await?;

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

/**
When this route don't get access to the group cache
*/
pub(super) async fn check_group_rank_by_fetch(app_id: AppId, group_id: GroupId, user_id: UserId, req_rank: i32) -> AppRes<()>
{
	let rank = get_user_rank(app_id, group_id, user_id).await?;

	if rank > req_rank {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupUserRank,
			"Wrong group rank for this action".to_string(),
			None,
		));
	}

	Ok(())
}

pub(super) async fn get_user_rank(app_id: AppId, group_id: GroupId, user_id: UserId) -> AppRes<i32>
{
	//language=SQL
	let sql = r"
SELECT `rank` 
FROM 
    sentc_group_user gu,
    sentc_group g
WHERE 
    group_id = ? AND 
    id = group_id AND 
    app_id = ? AND
    user_id = ?";

	let rank: Option<UserGroupRankCheck> = query_first(sql, set_params!(group_id, app_id, user_id)).await?;

	let rank = match rank {
		Some(r) => r,
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::GroupUserNotFound,
				"Group user not found".to_string(),
				None,
			))
		},
	};

	Ok(rank.0)
}

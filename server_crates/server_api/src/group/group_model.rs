use sentc_crypto_common::group::CreateData;
use sentc_crypto_common::{AppId, GroupId, UserId};
use uuid::Uuid;

use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::core::db::{exec_transaction, query_first, TransactionData};
use crate::core::get_time;
use crate::group::group_entities::UserGroupRankCheck;
use crate::set_params;

pub(super) async fn create(app_id: AppId, user_id: UserId, data: CreateData) -> AppRes<GroupId>
{
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
     time
     ) 
VALUES (?,?,?,?,?,?,?,?,?)";

	let encrypted_ephemeral_key: Option<String> = None;
	let encrypted_group_key_by_eph_key: Option<String> = None;

	let group_data_params = set_params!(
		group_key_id,
		group_id.to_string(),
		data.keypair_encrypt_alg,
		data.encrypted_private_group_key,
		data.public_group_key,
		data.group_key_alg,
		encrypted_ephemeral_key,
		encrypted_group_key_by_eph_key,
		time.to_string()
	);

	//insert he creator => rank = 0
	//language=SQL
	let sql_group_user = "INSERT INTO sentc_group_user (user_id, group_id, time, `rank`) VALUES (?,?,?,?)";
	let group_user_params = set_params!(user_id.to_string(), group_id.to_string(), time.to_string(), 0);

	let group_user_key_id = Uuid::new_v4().to_string();

	//language=SQL
	let sql_group_user_keys = r"
INSERT INTO sentc_group_user_keys 
    (
     k_id, 
     user_id, 
     encrypted_group_key, 
     encrypted_alg, 
     encrypted_group_key_key_id,
     time
     ) 
VALUES (?,?,?,?,?,?)";

	let group_user_keys_params = set_params!(
		group_user_key_id,
		user_id,
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

pub(super) async fn delete(app_id: AppId, group_id: GroupId, user_id: UserId) -> AppRes<()>
{
	check_group_rank(group_id.to_string(), user_id, 1).await?;

	//delete with app id to make sure the user is in the right group
	//language=SQL
	let sql = "DELETE FROM sentc_group WHERE id = ? AND app_id = ?";
	let delete_params = set_params!(group_id.to_string(), app_id.to_string());

	//delete the children
	//language=SQL
	let sql_delete_child = "DELETE FROM sentc_group WHERE parent = ? AND app_id = ?";
	let delete_children_params = set_params!(group_id, app_id);

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

	Ok(())
}

async fn check_group_rank(group_id: GroupId, user_id: UserId, req_rank: i32) -> AppRes<()>
{
	//language=SQL
	let sql = "SELECT `rank` FROM sentc_group_user WHERE group_id = ? AND user_id = ?";

	let rank: Option<UserGroupRankCheck> = query_first(sql.to_string(), set_params!(group_id, user_id)).await?;

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

	if rank.0 > req_rank {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupUserRank,
			"Wrong group rank for this action".to_string(),
			None,
		));
	}

	Ok(())
}

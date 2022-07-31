use sentc_crypto_common::group::{DoneKeyRotationData, GroupKeysForNewMember, KeyRotationData};
use sentc_crypto_common::{AppId, GroupId, SymKeyId, UserId};
use uuid::Uuid;

use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::core::db::{bulk_insert, exec, exec_transaction, query, query_first, TransactionData};
use crate::core::get_time;
use crate::group::group_entities::{
	GroupInviteReq,
	GroupJoinReq,
	GroupKeyUpdate,
	UserInGroupCheck,
	GROUP_INVITE_TYPE_INVITE_REQ,
	GROUP_INVITE_TYPE_JOIN_REQ,
};
use crate::group::group_model::check_group_rank;
use crate::set_params;

pub(super) async fn invite_request(
	group_id: GroupId,
	invited_user: UserId,
	keys_for_new_user: Vec<GroupKeysForNewMember>,
	admin_rank: i32,
) -> AppRes<()>
{
	//1. check the rights of the starter
	check_group_rank(admin_rank, 2)?;

	//2. check if the user is already in the group
	let check = check_user_in_group(group_id.to_string(), invited_user.to_string()).await?;

	if check == true {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupUserExists,
			"Invited user is already in the group".to_string(),
			None,
		));
	}

	//check if there was already an invite to this user -> don't use insert ignore here because we would insert the keys again!
	//language=SQL
	let sql = "SELECT 1 FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ? AND type = ?";
	let invite_exists: Option<UserInGroupCheck> = query_first(
		sql,
		set_params!(
			group_id.to_string(),
			invited_user.to_string(),
			GROUP_INVITE_TYPE_INVITE_REQ
		),
	)
	.await?;

	if invite_exists.is_some() {
		return Err(HttpErr::new(
			200,
			ApiErrorCodes::GroupUserExists,
			"User was already invited".to_string(),
			None,
		));
	}

	//______________________________________________________________________________________________

	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_group_user_invites_and_join_req (user_id, group_id, type, time) VALUES (?,?,?,?)";

	exec(
		sql,
		set_params!(
			invited_user.to_string(),
			group_id.to_string(),
			GROUP_INVITE_TYPE_INVITE_REQ,
			time.to_string()
		),
	)
	.await?;

	//insert the keys in the right table -> delete the keys from this table when user not accept the invite!
	bulk_insert(
		true,
		"sentc_group_user_keys".to_string(),
		vec![
			"user_id".to_string(),
			"k_id".to_string(),
			"group_id".to_string(),
			"encrypted_group_key".to_string(),
			"encrypted_group_key_key_id".to_string(),
			"encrypted_alg".to_string(),
		],
		keys_for_new_user,
		move |ob| {
			set_params!(
				invited_user.to_string(),
				ob.key_id.to_string(),
				group_id.to_string(),
				ob.encrypted_group_key.to_string(),
				ob.user_public_key_id.to_string(),
				ob.alg.to_string()
			)
		},
	)
	.await?;

	Ok(())
}

pub(super) async fn get_invite_req_to_user(app_id: AppId, user_id: UserId, last_fetched_time: u128) -> AppRes<Vec<GroupInviteReq>>
{
	//called from the user not the group

	//language=SQL
	let sql = "
SELECT group_id, i.time 
FROM sentc_group_user_invites_and_join_req i, sentc_group g 
WHERE 
    user_id = ? AND 
    i.time >= ? AND 
    type = ? AND 
    app_id = ? AND
    group_id = id
LIMIT 50";

	let invite_req: Vec<GroupInviteReq> = query(
		sql,
		set_params!(
			user_id,
			last_fetched_time.to_string(),
			GROUP_INVITE_TYPE_INVITE_REQ,
			app_id
		),
	)
	.await?;

	Ok(invite_req)
}

pub(super) async fn reject_invite(group_id: GroupId, user_id: UserId) -> AppRes<()>
{
	//check if there is an invite (this is important, because we delete the user keys too)
	check_for_invite(user_id.to_string(), group_id.to_string()).await?;

	//language=SQL
	let sql = "DELETE FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ?";
	let params_in = set_params!(group_id.to_string(), user_id.to_string());

	//delete the keys from the users table -> no trigger in the db

	//language=SQL
	let sql_keys = "DELETE FROM sentc_group_user_keys WHERE group_id = ? AND user_id = ?";
	let params_keys = set_params!(group_id, user_id);

	exec_transaction(vec![
		TransactionData {
			sql,
			params: params_in,
		},
		TransactionData {
			sql: sql_keys,
			params: params_keys,
		},
	])
	.await?;

	Ok(())
}

pub(super) async fn accept_invite(group_id: GroupId, user_id: UserId) -> AppRes<()>
{
	//called from the invited user
	check_for_invite(user_id.to_string(), group_id.to_string()).await?;

	//delete the old entry
	//language=SQL
	let sql_del = "DELETE FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ?";
	let params_del = set_params!(group_id.to_string(), user_id.to_string());

	//insert the user into the user group table, the keys are already there from the start invite
	let time = get_time()?;

	//language=SQL
	let sql_in = "INSERT INTO sentc_group_user (user_id, group_id, time, `rank`) VALUES (?,?,?,?)";
	let params_in = set_params!(user_id, group_id, time.to_string(), 4);

	exec_transaction(vec![
		TransactionData {
			sql: sql_del,
			params: params_del,
		},
		TransactionData {
			sql: sql_in,
			params: params_in,
		},
	])
	.await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn join_req(group_id: GroupId, user_id: UserId) -> AppRes<()>
{
	let check = check_user_in_group(group_id.to_string(), user_id.to_string()).await?;

	if check == true {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupUserExists,
			"User is already in the group".to_string(),
			None,
		));
	}

	let time = get_time()?;

	//org, sql query, but wont work on sqlite
	#[cfg(feature = "mysql")]
	//language=SQL
	let sql = "INSERT IGNORE INTO sentc_group_user_invites_and_join_req (user_id, group_id, type, time) VALUES (?,?,?,?)";

	#[cfg(feature = "sqlite")]
	let sql = "INSERT OR IGNORE INTO sentc_group_user_invites_and_join_req (user_id, group_id, type, time) VALUES (?,?,?,?)";

	exec(
		sql,
		set_params!(
			user_id.to_string(),
			group_id.to_string(),
			GROUP_INVITE_TYPE_JOIN_REQ,
			time.to_string()
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn reject_join_req(group_id: GroupId, user_id: UserId, admin_rank: i32) -> AppRes<()>
{
	//called from the group admin
	check_group_rank(admin_rank, 2)?;

	//language=SQL
	let sql = "DELETE FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ?";

	exec(sql, set_params!(group_id, user_id)).await?;

	Ok(())
}

pub(super) async fn accept_join_req(group_id: GroupId, user_id: UserId, keys_for_new_user: Vec<GroupKeysForNewMember>, admin_rank: i32)
	-> AppRes<()>
{
	check_group_rank(admin_rank, 2)?;

	//this check in important (see invite user req -> check if there is an invite). we would insert the keys even if the user is already member
	let check = check_user_in_group(group_id.to_string(), user_id.to_string()).await?;

	if check == true {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupUserExists,
			"Invited user is already in the group".to_string(),
			None,
		));
	}

	//check if the join req exists
	//language=SQL
	let sql = "SELECT 1 FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ? AND type = ?";
	let check: Option<UserInGroupCheck> = query_first(
		sql,
		set_params!(group_id.to_string(), user_id.to_string(), GROUP_INVITE_TYPE_JOIN_REQ),
	)
	.await?;

	if check.is_none() {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::GroupJoinReqNotFound,
			"Join request not found".to_string(),
			None,
		));
	}

	//______________________________________________________________________________________________

	let time = get_time()?;

	//language=SQL
	let sql_del = "DELETE FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ?";
	let params_del = set_params!(group_id.to_string(), user_id.to_string());

	//language=SQL
	let sql_in = "INSERT INTO sentc_group_user (user_id, group_id, time, `rank`) VALUES (?,?,?,?)";
	let params_in = set_params!(user_id.to_string(), group_id.to_string(), time.to_string(), 4);

	exec_transaction(vec![
		TransactionData {
			sql: sql_del,
			params: params_del,
		},
		TransactionData {
			sql: sql_in,
			params: params_in,
		},
	])
	.await?;

	bulk_insert(
		true,
		"sentc_group_user_keys".to_string(),
		vec![
			"user_id".to_string(),
			"k_id".to_string(),
			"group_id".to_string(),
			"encrypted_group_key".to_string(),
			"encrypted_group_key_key_id".to_string(),
			"encrypted_alg".to_string(),
		],
		keys_for_new_user,
		move |ob| {
			set_params!(
				user_id.to_string(),
				ob.key_id.to_string(),
				group_id.to_string(),
				ob.encrypted_group_key.to_string(),
				ob.user_public_key_id.to_string(),
				ob.alg.to_string()
			)
		},
	)
	.await?;

	Ok(())
}

pub(super) async fn get_join_req(group_id: GroupId, last_fetched_time: u128, admin_rank: i32) -> AppRes<Vec<GroupJoinReq>>
{
	check_group_rank(admin_rank, 2)?;

	//fetch the user with public key in a separate req, when the user decided to accept a join req
	//language=SQL
	let sql = "SELECT user_id, time FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND time >= ? AND type = ? LIMIT 50";
	let join_req: Vec<GroupJoinReq> = query(
		sql,
		set_params!(group_id, last_fetched_time.to_string(), GROUP_INVITE_TYPE_JOIN_REQ),
	)
	.await?;

	Ok(join_req)
}

//__________________________________________________________________________________________________

pub(super) async fn user_leave_group(group_id: GroupId, user_id: UserId, rank: i32) -> AppRes<()>
{
	//get the rank of the user -> check if there is only one admin (check also here if the user is in the group)
	if rank <= 1 {
		let only_admin = check_for_only_one_admin(group_id.to_string(), user_id.to_string()).await?;

		if only_admin == true {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::GroupOnlyOneAdmin,
				"Can't leave the group, because no other admin found in the group. Please update the rank of another user to admin".to_string(),
				None,
			));
		}
	}

	//language=SQL
	let sql = "DELETE FROM sentc_group_user WHERE group_id = ? AND user_id = ?";

	exec(sql, set_params!(group_id, user_id)).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn start_key_rotation(group_id: GroupId, user_id: UserId, input: KeyRotationData) -> AppRes<SymKeyId>
{
	//insert the new group key

	let key_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//language=SQL
	let sql = r"
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
     ephemeral_alg,
     time
     ) VALUES (?,?,?,?,?,?,?,?,?,?,?)";

	let params = set_params!(
		key_id.to_string(),
		group_id.to_string(),
		input.keypair_encrypt_alg,
		input.encrypted_private_group_key,
		input.public_group_key,
		input.group_key_alg,
		input.encrypted_ephemeral_key,
		input.encrypted_group_key_by_ephemeral,
		input.previous_group_key_id,
		input.ephemeral_alg,
		time.to_string()
	);

	//insert the rotated keys (from the starter) into the group user keys

	//language=SQL
	let sql_user = r"
INSERT INTO sentc_group_user_keys 
    (
     k_id, 
     user_id, 
     group_id, 
     encrypted_group_key, 
     encrypted_alg, 
     encrypted_group_key_key_id, 
     time
     ) VALUES (?,?,?,?,?,?,?)";

	let params_user = set_params!(
		key_id.to_string(),
		user_id,
		group_id,
		input.encrypted_group_key_by_user,
		input.encrypted_group_key_alg,
		input.invoker_public_key_id,
		time.to_string()
	);

	exec_transaction(vec![
		TransactionData {
			sql,
			params,
		},
		TransactionData {
			sql: sql_user,
			params: params_user,
		},
	])
	.await?;

	Ok(key_id)
}

pub(super) async fn get_keys_for_key_update(app_id: AppId, group_id: GroupId, user_id: UserId) -> AppRes<Vec<GroupKeyUpdate>>
{
	//check if there was a key rotation, fetch all rotation keys in the table
	//order by ASC is important because we may need the old group key to decrypt the newer eph key

	//language=SQL
	let sql = r"
SELECT 
    gkr.encrypted_ephemeral_key, 
    gkr.encrypted_eph_key_key_id,	-- the key id of the public key which was used to encrypt the eph key on the server
    encrypted_group_key_by_eph_key,
    previous_group_key_id,
    ephemeral_alg,
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

pub(super) async fn done_key_rotation_for_user(group_id: GroupId, user_id: UserId, key_id: SymKeyId, input: DoneKeyRotationData) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = r"
INSERT INTO sentc_group_user_keys 
    (
     k_id, 
     user_id, 
     group_id, 
     encrypted_group_key, 
     encrypted_alg, 
     encrypted_group_key_key_id, 
     time
     ) VALUES (?,?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			key_id.to_string(),
			user_id.to_string(),
			group_id.to_string(),
			input.encrypted_new_group_key,
			input.encrypted_alg,
			input.public_key_id,
			time.to_string()
		),
	)
	.await?;

	//delete the done keys -> do this after the insert, to make sure insert was successfully
	//language=SQL
	let sql = "DELETE FROM sentc_group_user_key_rotation WHERE group_id = ? AND user_id = ? AND key_id = ?";

	exec(sql, set_params!(group_id, user_id, key_id)).await?;

	Ok(())
}

//__________________________________________________________________________________________________

async fn check_user_in_group(group_id: GroupId, user_id: UserId) -> AppRes<bool>
{
	//language=SQL
	let sql = "SELECT 1 FROM sentc_group_user WHERE user_id = ? AND group_id = ? LIMIT 1";

	let exists: Option<UserInGroupCheck> = query_first(sql, set_params!(user_id, group_id)).await?;

	match exists {
		Some(_) => Ok(true),
		None => Ok(false),
	}
}

async fn check_for_invite(user_id: UserId, group_id: GroupId) -> AppRes<()>
{
	//language=SQL
	let sql = "SELECT 1 FROM sentc_group_user_invites_and_join_req WHERE user_id = ? AND group_id = ? AND type = ?";

	let check: Option<UserInGroupCheck> = query_first(sql, set_params!(user_id, group_id, GROUP_INVITE_TYPE_INVITE_REQ)).await?;

	match check {
		Some(_) => Ok(()),
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::GroupInviteNotFound,
				"No invite found".to_string(),
				None,
			))
		},
	}
}

/**
Used for leave group and change the own rank
*/
async fn check_for_only_one_admin(group_id: GroupId, user_id: UserId) -> AppRes<bool>
{
	//admin rank -> check if there is another admin. if not -> can't leave
	//language=SQL
	let sql = "SELECT 1 FROM sentc_group_user WHERE group_id = ? AND user_id NOT LIKE ? AND `rank` <= 1 LIMIT 1";

	let admin_found: Option<UserInGroupCheck> = query_first(sql, set_params!(group_id, user_id)).await?;

	//if there are more admins -> then the user is not the only admin
	match admin_found {
		Some(_) => Ok(false),
		None => Ok(true),
	}
}

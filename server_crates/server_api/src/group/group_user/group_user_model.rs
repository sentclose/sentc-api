use sentc_crypto_common::group::GroupKeysForNewMember;
use sentc_crypto_common::{AppId, GroupId, UserId};
use server_core::db::{bulk_insert, exec, exec_transaction, query_first, query_string, I32Entity, I64Entity, StringEntity, TransactionData};
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::res::AppRes;
use server_core::{get_time, set_params};
use uuid::Uuid;

use crate::group::group_entities::{GroupInviteReq, GroupJoinReq, GroupUserListItem, GROUP_INVITE_TYPE_INVITE_REQ, GROUP_INVITE_TYPE_JOIN_REQ};
use crate::group::group_model;
use crate::group::group_model::check_group_rank;
use crate::group::group_user_service::{InsertNewUserType, NewUserType};
use crate::util::api_res::ApiErrorCodes;

pub(super) async fn get_group_member(
	group_id: impl Into<GroupId>,
	user_id: impl Into<UserId>,
	last_fetched_time: u128,
	last_fetched_id: impl Into<UserId>,
) -> AppRes<Vec<GroupUserListItem>>
{
	//language=SQL
	let sql = r"
SELECT user_id, `rank`, time, type
FROM 
    sentc_group_user
WHERE 
    user_id NOT LIKE ? AND 
    group_id = ?"
		.to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND time >= ? AND (time > ?  OR (time = ? AND user_id > ?)) ORDER BY time, user_id LIMIT 50";
		(
			sql,
			set_params!(
				user_id.into(),
				group_id.into(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY time, user_id LIMIT 50";
		(sql, set_params!(user_id.into(), group_id.into()))
	};

	let list: Vec<GroupUserListItem> = query_string(sql, params).await?;

	Ok(list)
}

//__________________________________________________________________________________________________

pub(super) async fn invite_request(
	group_id: impl Into<GroupId>,
	invited_user: impl Into<UserId>,
	keys_for_new_user: Vec<GroupKeysForNewMember>,
	key_session: bool,
	admin_rank: i32,
	user_type: NewUserType,
) -> AppRes<Option<String>>
{
	let group_id = group_id.into();
	let invited_user = invited_user.into();

	//1. check the rights of the starter
	check_group_rank(admin_rank, 2)?;

	//2. get the int user type and if it is a group check if the group is a non connected group
	// do it in the model because we don't get any infos about the group until now
	let user_type = match user_type {
		NewUserType::Normal => 0,
		NewUserType::Group => {
			let cg = check_is_connected_group(invited_user.clone()).await?;

			if cg == 1 {
				return Err(SentcCoreError::new_msg(
					400,
					ApiErrorCodes::GroupJoinAsConnectedGroup,
					"Can't invite group when the group is a connected group",
				));
			}

			2
		},
	};

	//3. check if the user is already in the group
	let check = check_user_in_group(group_id.clone(), invited_user.clone()).await?;

	if check {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserExists,
			"Invited user is already in the group",
		));
	}

	//check if there was already an invite to this user -> don't use insert ignore here because we would insert the keys again!
	//language=SQL
	let sql = "SELECT 1 FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ? AND type = ?";
	let invite_exists: Option<I32Entity> = query_first(
		sql,
		set_params!(group_id.clone(), invited_user.clone(), GROUP_INVITE_TYPE_INVITE_REQ),
	)
	.await?;

	if invite_exists.is_some() {
		return Err(SentcCoreError::new_msg(
			200,
			ApiErrorCodes::GroupUserExists,
			"User was already invited",
		));
	}

	//______________________________________________________________________________________________

	let time = get_time()?;

	let (sql, params, session_id) = if key_session && keys_for_new_user.len() == 100 {
		//if there are more keys than 100 -> use a session,
		// the client will know if there are more keys than 100 and asks the server for a session
		let session_id = Uuid::new_v4().to_string();

		//language=SQL
		let sql_in = "INSERT INTO sentc_group_user_invites_and_join_req (user_id, group_id, type, time, key_upload_session_id, user_type) VALUES (?,?,?,?,?,?)";
		let params_in = set_params!(
			invited_user.clone(),
			group_id.clone(),
			GROUP_INVITE_TYPE_INVITE_REQ,
			time.to_string(),
			session_id.clone(),
			user_type
		);

		(sql_in, params_in, Some(session_id))
	} else {
		//language=SQL
		let sql_in = "INSERT INTO sentc_group_user_invites_and_join_req (user_id, group_id, type, time, user_type) VALUES (?,?,?,?,?)";
		let params_in = set_params!(
			invited_user.clone(),
			group_id.clone(),
			GROUP_INVITE_TYPE_INVITE_REQ,
			time.to_string(),
			user_type
		);

		(sql_in, params_in, None)
	};

	exec(sql, params).await?;

	insert_user_keys(group_id, invited_user, time, keys_for_new_user).await?;

	Ok(session_id)
}

pub(super) async fn get_invite_req_to_user(
	app_id: impl Into<AppId>,
	user_id: impl Into<UserId>,
	last_fetched_time: u128,
	last_id: impl Into<GroupId>,
) -> AppRes<Vec<GroupInviteReq>>
{
	//called from the user not the group

	//language=SQL
	let sql = "
SELECT group_id, i.time 
FROM sentc_group_user_invites_and_join_req i, sentc_group g 
WHERE 
    user_id = ? AND 
    i.type = ? AND 
    app_id = ? AND
    group_id = id"
		.to_string();

	let (sql1, params) = if last_fetched_time > 0 {
		//there is a last fetched time time -> set the last fetched time to the params list
		//order by time first -> then group id if multiple group ids got the same time
		let sql = sql + " AND i.time <= ? AND (i.time < ? OR (i.time = ? AND group_id > ?)) ORDER BY i.time DESC, group_id LIMIT 50";
		(
			sql,
			set_params!(
				user_id.into(),
				GROUP_INVITE_TYPE_INVITE_REQ,
				app_id.into(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY i.time DESC, group_id LIMIT 50";
		(
			sql,
			set_params!(user_id.into(), GROUP_INVITE_TYPE_INVITE_REQ, app_id.into()),
		)
	};

	let invite_req: Vec<GroupInviteReq> = query_string(sql1, params).await?;

	Ok(invite_req)
}

pub(super) async fn reject_invite(group_id: impl Into<GroupId>, user_id: impl Into<UserId>) -> AppRes<()>
{
	let group_id = group_id.into();
	let user_id = user_id.into();

	//check if there is an invite (this is important, because we delete the user keys too)
	check_for_invite(&user_id, &group_id).await?;

	//language=SQL
	let sql = "DELETE FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ?";
	let params_in = set_params!(group_id.clone(), user_id.clone());

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

pub(super) async fn accept_invite(group_id: impl Into<GroupId>, user_id: impl Into<UserId>) -> AppRes<()>
{
	let group_id = group_id.into();
	let user_id = user_id.into();

	//called from the invited user
	let user_type = check_for_invite(&user_id, &group_id).await?;

	//delete the old entry
	//language=SQL
	let sql_del = "DELETE FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ?";
	let params_del = set_params!(group_id.clone(), user_id.clone());

	//insert the user into the user group table, the keys are already there from the start invite
	let time = get_time()?;

	//language=SQL
	let sql_in = "INSERT INTO sentc_group_user (user_id, group_id, time, `rank`, type) VALUES (?,?,?,?,?)";
	let params_in = set_params!(user_id, group_id, time.to_string(), 4, user_type);

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

pub(super) async fn join_req(app_id: impl Into<AppId>, group_id: impl Into<GroupId>, user_id: impl Into<UserId>, user_type: NewUserType)
	-> AppRes<()>
{
	let group_id = group_id.into();
	let user_id = user_id.into();

	let user_type = match user_type {
		NewUserType::Normal => 0,
		NewUserType::Group => {
			//when it is a group wants to join another group -> check if the group to join is a connected group
			let cg = check_is_connected_group(&group_id).await?;

			if cg != 1 {
				return Err(SentcCoreError::new_msg(
					400,
					ApiErrorCodes::GroupJoinAsConnectedGroup,
					"Can't join a group when it is not a connected group",
				));
			}

			2
		},
	};

	let check = check_user_in_group(&group_id, &user_id).await?;

	if check {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserExists,
			"User is already in the group",
		));
	}

	//check if this group can be invited
	group_accept_invite(app_id, &group_id).await?;

	let time = get_time()?;

	//org, sql query, but wont work on sqlite
	#[cfg(feature = "mysql")]
	//language=SQL
	let sql = "INSERT IGNORE INTO sentc_group_user_invites_and_join_req (user_id, group_id, type, time, user_type) VALUES (?,?,?,?,?)";

	#[cfg(feature = "sqlite")]
	let sql = "INSERT OR IGNORE INTO sentc_group_user_invites_and_join_req (user_id, group_id, type, time, user_type) VALUES (?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			user_id,
			group_id,
			GROUP_INVITE_TYPE_JOIN_REQ,
			time.to_string(),
			user_type
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn reject_join_req(group_id: impl Into<GroupId>, user_id: impl Into<UserId>, admin_rank: i32) -> AppRes<()>
{
	//called from the group admin
	check_group_rank(admin_rank, 2)?;

	//language=SQL
	let sql = "DELETE FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ?";

	exec(sql, set_params!(group_id.into(), user_id.into())).await?;

	Ok(())
}

pub(super) async fn accept_join_req(
	group_id: impl Into<GroupId>,
	user_id: impl Into<UserId>,
	keys_for_new_user: Vec<GroupKeysForNewMember>,
	key_session: bool,
	admin_rank: i32,
) -> AppRes<Option<String>>
{
	let group_id = group_id.into();
	let user_id = user_id.into();

	check_group_rank(admin_rank, 2)?;

	//this check in important (see invite user req -> check if there is an invite). we would insert the keys even if the user is already member
	let check = check_user_in_group(group_id.clone(), user_id.clone()).await?;

	if check {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserExists,
			"Invited user is already in the group",
		));
	}

	//check if the join req exists
	//language=SQL
	let sql = "SELECT user_type FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ? AND type = ?";
	let check: Option<I32Entity> = query_first(
		sql,
		set_params!(group_id.clone(), user_id.clone(), GROUP_INVITE_TYPE_JOIN_REQ),
	)
	.await?;

	let user_type = match check {
		Some(c) => c.0,
		None => {
			return Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::GroupJoinReqNotFound,
				"Join request not found",
			));
		},
	};

	//______________________________________________________________________________________________

	let time = get_time()?;

	//language=SQL
	let sql_del = "DELETE FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND user_id = ?";
	let params_del = set_params!(group_id.clone(), user_id.clone());

	let (sql_in, params_in, session_id) = if key_session && keys_for_new_user.len() == 100 {
		//if there are more keys than 100 -> use a session,
		// the client will know if there are more keys than 100 and asks the server for a session
		let session_id = Uuid::new_v4().to_string();

		//language=SQL
		let sql_in = "INSERT INTO sentc_group_user (user_id, group_id, time, `rank`, key_upload_session_id, type) VALUES (?,?,?,?,?,?)";
		let params_in = set_params!(
			user_id.clone(),
			group_id.clone(),
			time.to_string(),
			4,
			session_id.clone(),
			user_type
		);

		(sql_in, params_in, Some(session_id))
	} else {
		//language=SQL
		let sql_in = "INSERT INTO sentc_group_user (user_id, group_id, time, `rank`, type) VALUES (?,?,?,?,?)";
		let params_in = set_params!(user_id.clone(), group_id.clone(), time.to_string(), 4, user_type);

		(sql_in, params_in, None)
	};

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

	insert_user_keys(group_id, user_id, time, keys_for_new_user).await?;

	Ok(session_id)
}

pub(super) async fn get_join_req(
	group_id: impl Into<GroupId>,
	last_fetched_time: u128,
	last_id: impl Into<UserId>,
	admin_rank: i32,
) -> AppRes<Vec<GroupJoinReq>>
{
	check_group_rank(admin_rank, 2)?;

	//language=SQL
	let sql = r"
SELECT user_id, time, user_type 
FROM sentc_group_user_invites_and_join_req 
WHERE group_id = ? AND type = ?"
		.to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND time >= ? AND (time > ? OR (time = ? AND user_id > ?)) ORDER BY time, user_id LIMIT 50";
		(
			sql,
			set_params!(
				group_id.into(),
				GROUP_INVITE_TYPE_JOIN_REQ,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY time, user_id LIMIT 50";
		(sql, set_params!(group_id.into(), GROUP_INVITE_TYPE_JOIN_REQ))
	};

	//fetch the user with public key in a separate req, when the user decided to accept a join req
	let join_req: Vec<GroupJoinReq> = query_string(sql, params).await?;

	Ok(join_req)
}

pub(super) async fn get_sent_join_req(
	app_id: impl Into<AppId>,
	user_id: impl Into<UserId>,
	last_fetched_time: u128,
	last_id: impl Into<GroupId>,
) -> AppRes<Vec<GroupInviteReq>>
{
	//the same as get_invite_req_to_user but with another search type: join req instead of invites

	//language=SQL
	let sql = r"
SELECT group_id, i.time 
FROM 
    sentc_group_user_invites_and_join_req i, 
    sentc_group g 
WHERE  
    user_id = ? AND 
    i.type = ? AND 
    app_id = ? AND
    group_id = id"
		.to_string();

	let (sql1, params) = if last_fetched_time > 0 {
		//there is a last fetched time time -> set the last fetched time to the params list
		//order by time first -> then group id if multiple group ids got the same time
		let sql = sql + " AND i.time <= ? AND (i.time < ? OR (i.time = ? AND group_id > ?)) ORDER BY i.time DESC, group_id LIMIT 50";
		(
			sql,
			set_params!(
				user_id.into(),
				GROUP_INVITE_TYPE_JOIN_REQ,
				app_id.into(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY i.time DESC, group_id LIMIT 50";
		(
			sql,
			set_params!(user_id.into(), GROUP_INVITE_TYPE_JOIN_REQ, app_id.into()),
		)
	};

	let invite_req: Vec<GroupInviteReq> = query_string(sql1, params).await?;

	Ok(invite_req)
}

pub(super) async fn delete_sent_join_req(app_id: impl Into<AppId>, user_id: impl Into<UserId>, group_id: impl Into<GroupId>) -> AppRes<()>
{
	let group_id = group_id.into();

	//check the app id extra because sqlite doesn't support multiple tables in delete from
	//language=SQL
	let sql = "SELECT 1 FROM sentc_group WHERE app_id = ? AND id = ?";

	let check: Option<I64Entity> = query_first(sql, set_params!(app_id.into(), group_id.clone())).await?;

	if check.is_none() {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupAccess,
			"Group not found",
		));
	}

	//language=SQL
	let sql = "DELETE FROM sentc_group_user_invites_and_join_req WHERE user_id = ? AND group_id = ?";

	exec(sql, set_params!(user_id.into(), group_id)).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn user_leave_group(group_id: impl Into<GroupId>, user_id: impl Into<UserId>, rank: i32) -> AppRes<()>
{
	let group_id = group_id.into();
	let user_id = user_id.into();

	//get the rank of the user -> check if there is only one admin (check also here if the user is in the group)
	if rank <= 1 {
		let only_admin = check_for_only_one_admin(group_id.clone(), user_id.clone()).await?;

		if only_admin {
			return Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::GroupOnlyOneAdmin,
				"Can't leave the group, because no other admin found in the group. Please update the rank of another user to admin",
			));
		}
	}

	//language=SQL
	let sql = "DELETE FROM sentc_group_user WHERE group_id = ? AND user_id = ? AND type NOT LIKE ?";

	//only delete normal user or group as member
	exec(sql, set_params!(group_id, user_id, 1)).await?;

	Ok(())
}

pub(super) async fn kick_user_from_group(group_id: impl Into<GroupId>, user_id: impl Into<UserId>, rank: i32) -> AppRes<()>
{
	let group_id = group_id.into();
	let user_id = user_id.into();

	check_group_rank(rank, 2)?;

	//check the rank of the member -> if it is the creator => don't kick

	//language=SQL
	let sql = "SELECT `rank` FROM sentc_group_user WHERE user_id = ? AND group_id = ?";

	let check: Option<I32Entity> = query_first(sql, set_params!(user_id.clone(), group_id.clone())).await?;

	let check = match check {
		Some(c) => c.0,
		None => {
			return Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::GroupUserNotFound,
				"User not found in this group",
			))
		},
	};

	if check == 0 {
		//changed user is creator
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserKick,
			"Can't delete the group creator",
		));
	}

	if check < rank {
		//changed user has a higher rank
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserKickRank,
			"Can't delete a higher rank.",
		));
	}

	//language=SQL
	let sql = "DELETE FROM sentc_group_user WHERE group_id = ? AND user_id = ?";

	exec(sql, set_params!(group_id, user_id)).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn update_rank(group_id: impl Into<GroupId>, admin_rank: i32, changed_user_id: impl Into<UserId>, new_rank: i32) -> AppRes<()>
{
	let group_id = group_id.into();
	let changed_user_id = changed_user_id.into();

	check_group_rank(admin_rank, 1)?;

	//only one creator
	if new_rank == 0 || new_rank > 4 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserRankUpdate,
			"Wrong rank used",
		));
	}

	//check if this user wants to cache the rank of the creator and check if the user exists in this group
	//language=SQL
	let sql = "SELECT `rank` FROM sentc_group_user WHERE user_id = ? AND group_id = ?";

	let check: Option<I32Entity> = query_first(sql, set_params!(changed_user_id.clone(), group_id.clone())).await?;

	let check = match check {
		Some(c) => c.0,
		None => {
			return Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::GroupUserNotFound,
				"User not found in this group",
			))
		},
	};

	if check == 0 {
		//changed user is creator
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupUserRankUpdate,
			"Can't change the rank of a group creator",
		));
	}

	//language=SQL
	let sql = "UPDATE sentc_group_user SET `rank` = ? WHERE group_id = ? AND user_id = ?";

	exec(sql, set_params!(new_rank, group_id, changed_user_id)).await?;

	Ok(())
}

//__________________________________________________________________________________________________

/**
Where there are too many keys used in this group.

Use session to upload the keys.
this session is automatically created when doing invite req or accepting join req
*/
pub(super) async fn insert_user_keys_via_session(
	group_id: impl Into<GroupId>,
	session_id: impl Into<String>,
	keys_for_new_user: Vec<GroupKeysForNewMember>,
	insert_type: InsertNewUserType,
) -> AppRes<()>
{
	let group_id = group_id.into();

	//check the session id
	let sql = match insert_type {
		InsertNewUserType::Invite => {
			//language=SQL
			"SELECT user_id FROM sentc_group_user_invites_and_join_req WHERE group_id = ? AND key_upload_session_id = ?"
		},
		InsertNewUserType::Join => {
			//language=SQL
			"SELECT user_id FROM sentc_group_user WHERE group_id = ? AND key_upload_session_id = ?"
		},
	};

	let user_id: Option<StringEntity> = query_first(sql, set_params!(group_id.clone(), session_id.into())).await?;
	let user_id = match user_id {
		Some(id) => id.0,
		None => {
			return Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::GroupKeySession,
				"No session found to upload the keys",
			))
		},
	};

	let time = get_time()?;

	insert_user_keys(group_id, user_id, time, keys_for_new_user).await?;

	Ok(())
}

//__________________________________________________________________________________________________

async fn insert_user_keys(
	group_id: impl Into<GroupId>,
	new_user_id: impl Into<UserId>,
	time: u128,
	keys_for_new_user: Vec<GroupKeysForNewMember>,
) -> AppRes<()>
{
	let group_id = group_id.into();
	let new_user_id = new_user_id.into();

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
			"time".to_string(),
		],
		keys_for_new_user,
		move |ob| {
			set_params!(
				new_user_id.clone(),
				ob.key_id.clone(),
				group_id.clone(),
				ob.encrypted_group_key.clone(),
				ob.user_public_key_id.clone(),
				ob.encrypted_alg.clone(),
				time.to_string()
			)
		},
	)
	.await?;

	Ok(())
}

async fn check_user_in_group(group_id: impl Into<GroupId>, user_id: impl Into<UserId>) -> AppRes<bool>
{
	let group_id = group_id.into();
	let user_id = user_id.into();

	//language=SQL
	let sql = "SELECT 1 FROM sentc_group_user WHERE user_id = ? AND group_id = ? LIMIT 1";

	let exists: Option<I32Entity> = query_first(sql, set_params!(user_id.clone(), group_id.clone())).await?;

	if exists.is_some() {
		return Ok(true);
	}

	//check if the user is in a parent group
	let exists = group_model::get_user_from_parent_groups(group_id, user_id).await?;

	match exists {
		Some(_) => Ok(true),
		None => Ok(false),
	}
}

async fn check_for_invite(user_id: impl Into<UserId>, group_id: impl Into<GroupId>) -> AppRes<i32>
{
	//language=SQL
	let sql = "SELECT user_type FROM sentc_group_user_invites_and_join_req WHERE user_id = ? AND group_id = ? AND type = ?";

	let check: Option<I32Entity> = query_first(
		sql,
		set_params!(user_id.into(), group_id.into(), GROUP_INVITE_TYPE_INVITE_REQ),
	)
	.await?;

	match check {
		Some(user_type) => Ok(user_type.0),
		None => {
			Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::GroupInviteNotFound,
				"No invite found",
			))
		},
	}
}

/**
Used for leave group and change the own rank
*/
async fn check_for_only_one_admin(group_id: impl Into<GroupId>, user_id: impl Into<UserId>) -> AppRes<bool>
{
	//admin rank -> check if there is another admin. if not -> can't leave
	//language=SQL
	let sql = "SELECT 1 FROM sentc_group_user WHERE group_id = ? AND user_id NOT LIKE ? AND `rank` <= 1 LIMIT 1";

	let admin_found: Option<I32Entity> = query_first(sql, set_params!(group_id.into(), user_id.into())).await?;

	//if there are more admins -> then the user is not the only admin
	match admin_found {
		Some(_) => Ok(false),
		None => Ok(true),
	}
}

#[inline(always)]
async fn group_accept_invite(app_id: impl Into<AppId>, group_id: impl Into<GroupId>) -> AppRes<()>
{
	//check if this group can be invited
	//language=SQL
	let sql = "SELECT invite FROM sentc_group WHERE app_id = ? AND id = ?";
	let can_invite: Option<I32Entity> = query_first(sql, set_params!(app_id.into(), group_id.into())).await?;

	match can_invite {
		Some(ci) => {
			if ci.0 == 0 {
				return Err(SentcCoreError::new_msg(
					400,
					ApiErrorCodes::GroupInviteStop,
					"No invites allowed for this group",
				));
			}
		},
		None => {
			return Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::GroupAccess,
				"Group not found",
			))
		},
	}

	Ok(())
}

async fn check_is_connected_group(group_id: impl Into<GroupId>) -> AppRes<i32>
{
	//language=SQL
	let sql = "SELECT is_connected_group FROM sentc_group WHERE id = ?";
	let is_connected_group: Option<I32Entity> = query_first(sql, set_params!(group_id.into())).await?;

	if let Some(cg) = is_connected_group {
		Ok(cg.0)
	} else {
		Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupAccess,
			"Group to invite not found",
		))
	}
}

use sentc_crypto_common::group::GroupKeysForNewMember;
use sentc_crypto_common::{AppId, GroupId, UserId};

use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::core::db::{bulk_insert, exec, exec_transaction, query_first, TransactionData};
use crate::core::get_time;
use crate::group::group_entities::{UserGroupRankCheck, GROUP_INVITE_TYPE_INVITE_REQ};
use crate::group::group_model::{check_group_rank, check_user_in_group};
use crate::set_params;

pub(super) async fn invite_request(
	app_id: AppId,
	group_id: GroupId,
	starter_user_id: UserId,
	invited_user: UserId,
	keys_for_new_user: Vec<GroupKeysForNewMember>,
) -> AppRes<()>
{
	//1. check the rights of the starter
	check_group_rank(app_id, group_id.to_string(), starter_user_id.to_string(), 2).await?;

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

pub(super) async fn user_leave_group(app_id: AppId, group_id: GroupId, user_id: UserId) -> AppRes<()>
{
	//TODO
	//get the rank of the user -> check if there is only one admin

	//language=SQL
	let sql = r"
DELETE sentc_group_user 
FROM 
    sentc_group_user,
    sentc_group 
WHERE 
    group_id = ? AND 
    user_id = ? AND 
    app_id = ? AND 
    id = group_id";

	exec(sql, set_params!(group_id, user_id, app_id)).await?;

	Ok(())
}

async fn check_for_invite(user_id: UserId, group_id: GroupId) -> AppRes<()>
{
	//language=SQL
	let sql = "SELECT 1 FROM sentc_group_user_invites_and_join_req WHERE user_id = ? AND group_id = ?";

	let check: Option<UserGroupRankCheck> = query_first(sql.to_string(), set_params!(user_id, group_id)).await?;

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

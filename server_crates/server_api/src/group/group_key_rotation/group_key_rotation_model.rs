use rustgram_server_util::db::id_handling::{check_id_format, create_id};
use rustgram_server_util::db::{bulk_insert, exec, exec_transaction, query, query_first, query_string, TransactionData};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::{get_time, set_params};
use sentc_crypto_common::group::{DoneKeyRotationData, KeyRotationData};
use sentc_crypto_common::{AppId, DeviceId, GroupId, SymKeyId, UserId};

use crate::group::group_entities::{GroupKeyUpdate, KeyRotationWorkerKey, UserEphKeyOut, UserGroupPublicKeyData};
use crate::util::api_res::ApiErrorCodes;

pub(super) async fn start_key_rotation(
	app_id: impl Into<AppId>,
	group_id: impl Into<GroupId>,
	user_id: impl Into<UserId>,
	input: KeyRotationData,
) -> AppRes<SymKeyId>
{
	check_id_format(&input.previous_group_key_id)?;
	check_id_format(&input.invoker_public_key_id)?;

	if let (Some(s_id), Some(s_sign_id)) = (&input.signed_by_user_id, &input.signed_by_user_sign_key_id) {
		check_id_format(s_id)?;
		check_id_format(s_sign_id)?;
	}

	let group_id = group_id.into();

	//insert the new group key

	let key_id = create_id();
	let time = get_time()?;

	//language=SQL
	let sql = r"
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
     ephemeral_alg,
     time,
     encrypted_sign_key,
     verify_key,
     keypair_sign_alg,
     signed_by_user_id,
     signed_by_user_sign_key_id,
     group_key_sig,
     public_key_sig,
     public_key_sig_key_id
     ) 
VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

	let params = set_params!(
		key_id.clone(),
		group_id.clone(),
		app_id.into(),
		input.keypair_encrypt_alg,
		input.encrypted_private_group_key,
		input.public_group_key,
		input.group_key_alg,
		input.encrypted_ephemeral_key,
		input.encrypted_group_key_by_ephemeral,
		input.previous_group_key_id,
		input.ephemeral_alg,
		time.to_string(),
		input.encrypted_sign_key,
		input.verify_key,
		input.keypair_sign_alg,
		input.signed_by_user_id,
		input.signed_by_user_sign_key_id,
		input.group_key_sig,
		input.public_key_sig,
		key_id.clone(),
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
		key_id.clone(),
		user_id.into(),
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

pub async fn get_keys_for_key_update(
	app_id: impl Into<AppId>,
	group_id: impl Into<GroupId>,
	user_id: impl Into<UserId>,
) -> AppRes<Vec<GroupKeyUpdate>>
{
	//check if there was a key rotation, fetch all rotation keys in the table
	//order by ASC is important because we may need the old group key to decrypt the newer eph key

	//language=SQL
	let sql = r"
SELECT 
    gk.id,
    error,
    gkr.encrypted_ephemeral_key,
    gkr.encrypted_eph_key_key_id,	-- the key id of the public key which was used to encrypt the eph key on the server
    encrypted_group_key_by_eph_key,
    previous_group_key_id,
    ephemeral_alg,
    gk.time
FROM 
    sentc_group_keys gk, 
    sentc_group_user_key_rotation gkr
WHERE user_id = ? AND 
      gk.group_id = ? AND 
      app_id = ? AND 
      key_id = gk.id
ORDER BY gk.time";

	query(sql, set_params!(user_id.into(), group_id.into(), app_id.into())).await
}

pub async fn done_key_rotation_for_user(
	group_id: impl Into<GroupId>,
	user_id: impl Into<UserId>,
	key_id: impl Into<SymKeyId>,
	input: DoneKeyRotationData,
) -> AppRes<()>
{
	let key_id = key_id.into();
	let user_id = user_id.into();
	let group_id = group_id.into();

	let time = get_time()?;

	#[cfg(feature = "mysql")]
	//language=SQL
	let sql = r"
INSERT IGNORE INTO sentc_group_user_keys 
    (
     k_id, 
     user_id, 
     group_id, 
     encrypted_group_key, 
     encrypted_alg, 
     encrypted_group_key_key_id, 
     time
     ) VALUES (?,?,?,?,?,?,?)";

	#[cfg(feature = "sqlite")]
	let sql = r"
INSERT OR IGNORE INTO sentc_group_user_keys 
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
			key_id.clone(),
			user_id.clone(),
			group_id.clone(),
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
//Worker

pub(super) async fn get_new_key(group_id: impl Into<GroupId>, key_id: impl Into<SymKeyId>) -> AppRes<KeyRotationWorkerKey>
{
	//language=SQL
	let sql = "SELECT ephemeral_alg,encrypted_ephemeral_key FROM sentc_group_keys WHERE group_id = ? AND id = ?";

	query_first(sql, set_params!(group_id.into(), key_id.into()))
		.await?
		.ok_or_else(|| {
			ServerCoreError::new_msg(
				400,
				ApiErrorCodes::GroupKeyRotationKeysNotFound,
				"Internal error, no group keys found, please try again",
			)
		})
}

pub(super) async fn get_user_and_public_key(
	group_id: impl Into<GroupId>,
	key_id: impl Into<SymKeyId>,
	last_fetched_time: u128,
	last_id: impl Into<UserId>,
) -> AppRes<Vec<UserGroupPublicKeyData>>
{
	let key_id = key_id.into();

	//get the public key from the user group
	//language=SQL
	let sql = r"
SELECT user_id, uk.id, public_key, private_key_pair_alg, gu.time
FROM 
    sentc_group_user gu,
    (
        -- get only the latest key of the user, list here every user with his/her latest public key
        SELECT MAX(ugk.time), ugk.id, u.id as id_user, public_key, private_key_pair_alg
        FROM 
            sentc_user u, 
            sentc_group_keys ugk
        WHERE 
            user_group_id = ugk.group_id
        GROUP BY ugk.group_id
    ) uk
WHERE 
    gu.user_id = uk.id_user AND
    gu.type = 0 AND -- only real user
    gu.group_id = ? AND
    NOT EXISTS(
        -- this user is already done -> skip
        SELECT 1 
        FROM sentc_group_user_keys gk 
        WHERE 
            gk.k_id = ? AND 
            gk.user_id = gu.user_id AND 
            gk.group_id = gu.group_id
    ) AND 
    NOT EXISTS(
        -- this user already got a key rotation but needs to done it on the client -> skip
        SELECT 1 
        FROM sentc_group_user_key_rotation grk 
        WHERE 
            grk.key_id = ? AND 
            grk.user_id = gu.user_id AND 
            grk.group_id = gu.group_id
    )"
	.to_string();

	let (sql1, params) = if last_fetched_time > 0 {
		//there is a last fetched time
		let sql = sql + " AND gu.time <= ? AND (gu.time < ? OR (gu.time = ? AND gu.user_id > ?)) ORDER BY gu.time DESC, gu.user_id LIMIT 100";
		(
			sql,
			set_params!(
				group_id.into(),
				key_id.clone(),
				key_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY gu.time DESC, gu.user_id LIMIT 100";
		(sql, set_params!(group_id.into(), key_id.clone(), key_id))
	};

	let users: Vec<UserGroupPublicKeyData> = query_string(sql1, params).await?;

	Ok(users)
}

pub(super) async fn get_group_as_member_public_key(
	group_id: impl Into<GroupId>,
	key_id: impl Into<SymKeyId>,
	last_fetched_time: u128,
	last_id: impl Into<GroupId>,
) -> AppRes<Vec<UserGroupPublicKeyData>>
{
	let key_id = key_id.into();

	//get here the public key data from a group as member

	//language=SQL
	let sql = r"
SELECT k.group_id, k.id, public_key, private_key_pair_alg, gu.time
FROM 
    sentc_group_user gu, 
    (
        SELECT MAX(time), group_id, id, public_key, private_key_pair_alg, time
        FROM sentc_group_keys
        GROUP BY group_id
    ) k
WHERE 
    gu.user_id = k.group_id AND 
    gu.type = 2 AND 
    gu.group_id = ? AND 
    NOT EXISTS(
        SELECT 1 
        FROM sentc_group_user_keys gk 
        WHERE 
            gk.k_id = ? AND 
            gk.user_id = gu.user_id AND 
            gk.group_id = gu.group_id
    ) AND 
    NOT EXISTS(
        SELECT 1 
        FROM sentc_group_user_key_rotation grk 
        WHERE 
            grk.key_id = ? AND 
            grk.user_id = gu.user_id AND 
            grk.group_id = gu.group_id
    )"
	.to_string();

	let (sql1, params) = if last_fetched_time > 0 {
		//there is a last fetched time
		let sql = sql + " AND gu.time <= ? AND (gu.time < ? OR (gu.time = ? AND gu.user_id > ?)) ORDER BY gu.time DESC, gu.user_id LIMIT 100";
		(
			sql,
			set_params!(
				group_id.into(),
				key_id.clone(),
				key_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY gu.time DESC, gu.user_id LIMIT 100";
		(sql, set_params!(group_id.into(), key_id.clone(), key_id))
	};

	let keys: Vec<UserGroupPublicKeyData> = query_string(sql1, params).await?;

	Ok(keys)
}

pub(super) async fn get_parent_group_and_public_key(
	group_id: impl Into<GroupId>,
	key_id: impl Into<SymKeyId>,
) -> AppRes<Option<UserGroupPublicKeyData>>
{
	let key_id = key_id.into();

	//no pagination needed because there is only one parent group
	//k.time is not needed here for parent but is needed for the entity (which is also needed for the key rotation fn)

	//language=SQL
	let sql = r"
SELECT parent, k.id, public_key, private_key_pair_alg, k.time 
FROM sentc_group g, sentc_group_keys k
WHERE 
    parent = group_id AND 
    g.id = ? AND 
    NOT EXISTS(
        SELECT 1
        FROM sentc_group_user_keys gk
        WHERE 
            gk.k_id = ? AND 
            user_id = parent
    ) AND 
    NOT EXISTS(
        SELECT 1
        FROM sentc_group_user_key_rotation grk
        WHERE 
            key_id = ? AND 
            user_id = parent
    )
ORDER BY k.time DESC 
LIMIT 1
";

	query_first(sql, set_params!(group_id.into(), key_id.clone(), key_id)).await
}

pub(super) async fn get_device_keys(
	user_id: impl Into<UserId>,
	key_id: impl Into<SymKeyId>,
	last_fetched_time: u128,
	last_id: impl Into<DeviceId>,
) -> AppRes<Vec<UserGroupPublicKeyData>>
{
	let key_id = key_id.into();

	//device keys for user key rotation

	//language=SQL
	let sql = r"
SELECT ud.id as device_id, ud.id as key_id, public_key, keypair_encrypt_alg, ud.time 
FROM sentc_user_device ud, sentc_user u
WHERE 
    user_id = u.id AND 
    user_id = ? AND 
    NOT EXISTS(
          -- this device is done -> skip
          SELECT  1 FROM sentc_group_user_keys gk
          WHERE 
              gk.k_id = ? AND 
              gk.user_id = ud.id AND 
              group_id = u.user_group_id
    ) AND 
    NOT EXISTS(
        -- this device got a rotation, but needs to done it in the client -> skip
        SELECT 1 FROM sentc_group_user_key_rotation grk
        WHERE 
            grk.key_id = ? AND 
            grk.user_id = ud.id AND 
            grk.group_id = u.user_group_id
    )"
	.to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND ud.time <= ? AND (ud.time < ? OR (ud.time = ? AND ud.id > ?)) ORDER BY ud.time DESC, ud.id LIMIT 100";
		(
			sql,
			set_params!(
				user_id.into(),
				key_id.clone(),
				key_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY ud.time DESC, ud.id LIMIT 100";
		(sql, set_params!(user_id.into(), key_id.clone(), key_id))
	};

	query_string(sql, params).await
}

pub(super) async fn save_user_eph_keys(group_id: impl Into<GroupId>, key_id: impl Into<SymKeyId>, keys: Vec<UserEphKeyOut>) -> AppRes<()>
{
	let group_id = group_id.into();
	let key_id = key_id.into();

	bulk_insert(
		true,
		"sentc_group_user_key_rotation",
		&[
			"key_id",
			"group_id",
			"user_id",
			"encrypted_ephemeral_key",
			"encrypted_eph_key_key_id",
			"error",
		],
		keys,
		move |ob| {
			set_params!(
				key_id.clone(),
				group_id.clone(),
				ob.user_id,
				ob.encrypted_ephemeral_key,
				ob.encrypted_eph_key_key_id,
				ob.rotation_err
			)
		},
	)
	.await?;

	Ok(())
}

/**
Do this after the key rotation.

delete just the eph key which was encrypted by the previous group key.
After the key rotation, this key is encrypted by every group members public key
and can't be reconstructed by just knowing this key and the previous group key
*/
pub(super) async fn delete_eph_key(group_id: impl Into<GroupId>, key_id: impl Into<SymKeyId>) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_group_keys SET encrypted_ephemeral_key = NULL WHERE group_id = ? AND id = ?";

	exec(sql, set_params!(group_id.into(), key_id.into())).await?;

	Ok(())
}

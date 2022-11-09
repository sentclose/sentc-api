use sentc_crypto_common::group::{DoneKeyRotationData, KeyRotationData};
use sentc_crypto_common::{AppId, DeviceId, EncryptionKeyPairId, GroupId, SymKeyId, UserId};
use server_core::db::{bulk_insert, exec, exec_transaction, query, query_first, query_string, TransactionData};
use server_core::{get_time, set_params};
use uuid::Uuid;

use crate::group::group_entities::{GroupKeyUpdate, KeyRotationWorkerKey, UserEphKeyOut, UserGroupPublicKeyData};
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

pub(super) async fn start_key_rotation(app_id: AppId, group_id: GroupId, user_id: UserId, input: KeyRotationData) -> AppRes<SymKeyId>
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
     keypair_sign_alg
     ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

	let params = set_params!(
		key_id.to_string(),
		group_id.to_string(),
		app_id,
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
		input.keypair_sign_alg
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
    gk.id,
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
//Worker

pub(super) async fn get_new_key(group_id: GroupId, key_id: SymKeyId) -> AppRes<KeyRotationWorkerKey>
{
	//language=SQL
	let sql = "SELECT ephemeral_alg,encrypted_ephemeral_key FROM sentc_group_keys WHERE group_id = ? AND id = ?";

	let key: Option<KeyRotationWorkerKey> = query_first(sql, set_params!(group_id, key_id)).await?;

	match key {
		Some(k) => Ok(k),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::GroupKeyRotationKeysNotFound,
				"Internal error, no group keys found, please try again".to_string(),
				None,
			))
		},
	}
}

pub(super) async fn get_user_and_public_key(
	group_id: GroupId,
	key_id: SymKeyId,
	last_fetched: u128,
	last_id: UserId,
) -> AppRes<Vec<UserGroupPublicKeyData>>
{
	//get the public key from the user group
	//language=SQL
	let sql = r"
SELECT user_id, ugk.id, public_key, private_key_pair_alg, gu.time
FROM 
    sentc_group_user gu, 
    sentc_group_keys ugk, 
    sentc_user u 
WHERE 
    gu.user_id = u.id AND 
    user_group_id = ugk.group_id AND 
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

	let (sql1, params) = if last_fetched > 0 {
		//there is a last fetched time time
		let sql = sql + " AND gu.time <= ? AND (gu.time < ? OR (gu.time = ? AND gu.user_id > ?)) ORDER BY gu.time DESC, gu.user_id LIMIT 100";
		(
			sql,
			set_params!(
				group_id,
				key_id.clone(),
				key_id,
				last_fetched.to_string(),
				last_fetched.to_string(),
				last_fetched.to_string(),
				last_id
			),
		)
	} else {
		let sql = sql + " ORDER BY gu.time DESC, gu.user_id LIMIT 100";
		(sql, set_params!(group_id, key_id.clone(), key_id))
	};

	let users: Vec<UserGroupPublicKeyData> = query_string(sql1, params).await?;

	Ok(users)
}

pub(super) async fn get_group_as_member_public_key(
	group_id: GroupId,
	key_id: SymKeyId,
	last_fetched: u128,
	last_id: GroupId,
) -> AppRes<Vec<UserGroupPublicKeyData>>
{
	//get here the public key data from a group as member

	//language=SQL
	let sql = r"
SELECT k.group_id, k.id, public_key, private_key_pair_alg, k.time
FROM 
    sentc_group_user gu, 
    sentc_group_keys k 
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

	let (sql1, params) = if last_fetched > 0 {
		//there is a last fetched time time
		let sql = sql + " AND gu.time <= ? AND (gu.time < ? OR (gu.time = ? AND gu.user_id > ?)) ORDER BY gu.time DESC, gu.user_id LIMIT 100";
		(
			sql,
			set_params!(
				group_id,
				key_id.clone(),
				key_id,
				last_fetched.to_string(),
				last_fetched.to_string(),
				last_fetched.to_string(),
				last_id
			),
		)
	} else {
		let sql = sql + " ORDER BY gu.time DESC, gu.user_id LIMIT 100";
		(sql, set_params!(group_id, key_id.clone(), key_id))
	};

	let keys: Vec<UserGroupPublicKeyData> = query_string(sql1, params).await?;

	Ok(keys)
}

pub(super) async fn get_parent_group_and_public_key(group_id: GroupId, key_id: SymKeyId) -> AppRes<Option<UserGroupPublicKeyData>>
{
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

	let keys: Option<UserGroupPublicKeyData> = query_first(sql, set_params!(group_id, key_id.to_string(), key_id)).await?;

	Ok(keys)
}

pub(super) async fn get_device_keys(user_id: UserId, key_id: SymKeyId, last_fetched: u128, last_id: DeviceId) -> AppRes<Vec<UserGroupPublicKeyData>>
{
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

	let (sql, params) = if last_fetched > 0 {
		let sql = sql + " AND ud.time <= ? AND (ud.time < ? OR (ud.time = ? AND ud.id > ?)) ORDER BY ud.time DESC, ud.id LIMIT 100";
		(
			sql,
			set_params!(
				user_id,
				key_id.clone(),
				key_id,
				last_fetched.to_string(),
				last_fetched.to_string(),
				last_fetched.to_string(),
				last_id
			),
		)
	} else {
		let sql = sql + " ORDER BY ud.time DESC, ud.id LIMIT 100";
		(sql, set_params!(user_id, key_id.clone(), key_id))
	};

	let devices: Vec<UserGroupPublicKeyData> = query_string(sql, params).await?;

	Ok(devices)
}

pub(super) async fn save_user_eph_keys(group_id: GroupId, key_id: EncryptionKeyPairId, keys: Vec<UserEphKeyOut>) -> AppRes<()>
{
	bulk_insert(
		true,
		"sentc_group_user_key_rotation".to_string(),
		vec![
			"key_id".to_string(),
			"group_id".to_string(),
			"user_id".to_string(),
			"encrypted_ephemeral_key".to_string(),
			"encrypted_eph_key_key_id".to_string(),
		],
		keys,
		move |ob| {
			set_params!(
				key_id.to_string(),
				group_id.to_string(),
				ob.user_id.to_string(),
				ob.encrypted_ephemeral_key.to_string(),
				ob.encrypted_eph_key_key_id.to_string()
			)
		},
	)
	.await?;

	Ok(())
}

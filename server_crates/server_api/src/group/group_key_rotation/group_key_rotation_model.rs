use sentc_crypto_common::group::{DoneKeyRotationData, KeyRotationData};
use sentc_crypto_common::{AppId, EncryptionKeyPairId, GroupId, SymKeyId, UserId};
use uuid::Uuid;

use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::core::db::{bulk_insert, exec, exec_transaction, query, query_first, query_string, TransactionData};
use crate::core::get_time;
use crate::group::group_entities::{GroupKeyUpdate, KeyRotationWorkerKey, UserEphKeyOut, UserGroupPublicKeyData};
use crate::set_params;

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

pub(super) async fn get_user_and_public_key(group_id: GroupId, last_fetched: u128) -> AppRes<Vec<UserGroupPublicKeyData>>
{
	//language=SQL
	let sql = r"
SELECT gu.user_id, public_key, id, keypair_encrypt_alg, gu.time 
FROM sentc_group_user gu, user_keys uk 
WHERE 
    gu.user_id = uk.user_id AND 
    group_id = ?"
		.to_string();

	let (sql1, params) = if last_fetched > 0 {
		//there is a last fetched time time
		let sql = sql + " AND gu.time <= ? ORDER BY gu.time DESC LIMIT 50";
		(sql, set_params!(group_id, last_fetched.to_string()))
	} else {
		let sql = sql + " ORDER BY gu.time DESC LIMIT 50";
		(sql, set_params!(group_id))
	};

	let users: Vec<UserGroupPublicKeyData> = query_string(sql1, params).await?;

	Ok(users)
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

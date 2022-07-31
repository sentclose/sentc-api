use sentc_crypto_common::group::{DoneKeyRotationData, KeyRotationData};
use sentc_crypto_common::{AppId, GroupId, SymKeyId, UserId};
use uuid::Uuid;

use crate::core::api_res::AppRes;
use crate::core::db::{exec, exec_transaction, query, TransactionData};
use crate::core::get_time;
use crate::group::group_entities::GroupKeyUpdate;
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

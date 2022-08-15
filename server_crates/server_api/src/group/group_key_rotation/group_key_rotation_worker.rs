use std::sync::Arc;

use sentc_crypto_common::{GroupId, SymKeyId};

use crate::group::group_entities::{KeyRotationWorkerKey, UserEphKeyOut, UserGroupPublicKeyData};
use crate::group::group_key_rotation::group_key_rotation_model;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

pub async fn start(group_id: GroupId, key_id: SymKeyId) -> AppRes<()>
{
	let key = group_key_rotation_model::get_new_key(group_id.to_string(), key_id.to_string()).await?;

	let key_arc = Arc::new(key);

	let mut last_time_fetched = 0;
	let mut last_user_id = "".to_string();

	loop {
		let key_cap = key_arc.clone();

		let users = group_key_rotation_model::get_user_and_public_key(
			group_id.to_string(),
			key_id.to_string(),
			last_time_fetched,
			last_user_id.to_string(),
		)
		.await?;
		let len = users.len();

		if len == 0 {
			break;
		}

		last_time_fetched = users[len - 1].time; //the last user is the oldest (order by time DESC)
		last_user_id = users[len - 1].user_id.to_string();

		//encrypt for each user
		let user_keys = tokio::task::spawn_blocking(move || encrypt(&key_cap, users))
			.await
			.map_err(|e| {
				HttpErr::new(
					400,
					ApiErrorCodes::GroupKeyRotationThread,
					"Error in user key rotation".to_string(),
					Some(format!("error in user key rotation: {}", e)),
				)
			})??;

		//save the keys for the user
		group_key_rotation_model::save_user_eph_keys(group_id.to_string(), key_id.to_string(), user_keys).await?;

		if len < 100 {
			//when there were less than 50 users in this fetch
			break;
		}
	}

	//key rotation for parent group. check first if this is already done for parent group (like user)
	match group_key_rotation_model::get_parent_group_and_public_key(group_id.to_string(), key_id.to_string()).await? {
		Some(item) => {
			let user_keys = tokio::task::spawn_blocking(move || encrypt(&key_arc, vec![item]))
				.await
				.map_err(|e| {
					HttpErr::new(
						400,
						ApiErrorCodes::GroupKeyRotationThread,
						"Error in user key rotation".to_string(),
						Some(format!("error in user key rotation: {}", e)),
					)
				})??;

			//save the keys for the parent
			group_key_rotation_model::save_user_eph_keys(group_id, key_id, user_keys).await?;
		},
		//no parent group found or the parent group is already done (e.g. was rotation starter)
		None => {},
	}

	Ok(())
}

fn encrypt(eph_key: &KeyRotationWorkerKey, users: Vec<UserGroupPublicKeyData>) -> AppRes<Vec<UserEphKeyOut>>
{
	let mut encrypted_keys: Vec<UserEphKeyOut> = Vec::with_capacity(users.len());

	for user in users {
		//encrypt with sdk -> import public key data from string

		let encrypted_ephemeral_key = sentc_crypto::util::server::encrypt_ephemeral_group_key_with_public_key(
			user.public_key.as_str(),
			user.public_key_alg.as_str(),
			eph_key.encrypted_ephemeral_key.as_str(),
		)
		.map_err(|_e| {
			HttpErr::new(
				400,
				ApiErrorCodes::GroupKeyRotationUserEncrypt,
				"Error in user key rotation encryption".to_string(),
				None,
			)
		})?;

		let ob = UserEphKeyOut {
			user_id: user.user_id,
			encrypted_ephemeral_key,
			encrypted_eph_key_key_id: user.public_key_id,
		};
		encrypted_keys.push(ob);
	}

	Ok(encrypted_keys)
}

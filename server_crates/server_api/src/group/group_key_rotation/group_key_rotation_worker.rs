use std::sync::Arc;

use sentc_crypto_common::{AppId, GroupId, SymKeyId};

use crate::group::group_entities::{KeyRotationWorkerKey, UserEphKeyOut, UserGroupPublicKeyData};
use crate::group::group_key_rotation::group_key_rotation_model;
use crate::user::user_service;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

pub async fn start(app_id: AppId, group_id: GroupId, key_id: SymKeyId, user_group: Option<String>) -> AppRes<()>
{
	let key = group_key_rotation_model::get_new_key(group_id.clone(), key_id.clone()).await?;

	let key_arc = Arc::new(key);

	let mut last_time_fetched = 0;
	let mut last_user_id = "".to_string();

	//track how many users are effected by the rotation
	let mut total_len = 0_usize;

	loop {
		let key_cap = key_arc.clone();

		let users = match &user_group {
			None => {
				group_key_rotation_model::get_user_and_public_key(
					group_id.clone(),
					key_id.clone(),
					last_time_fetched,
					last_user_id.clone(),
				)
				.await?
			},
			Some(u_id) => {
				//for a user group key rotation use the device id as user id and as public key id
				group_key_rotation_model::get_device_keys(u_id.clone(), key_id.clone(), last_time_fetched, last_user_id.clone()).await?
			},
		};

		let len = users.len();

		total_len += len;

		if len == 0 {
			break;
		}

		last_time_fetched = users[len - 1].time; //the last user is the oldest (order by time DESC)
		last_user_id = users[len - 1].user_id.clone();

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
			})?;

		//save the keys for the user
		group_key_rotation_model::save_user_eph_keys(group_id.clone(), key_id.clone(), user_keys).await?;

		if len < 100 {
			//when there were less than 50 users in this fetch
			break;
		}
	}

	//key rotation for parent group. check first if this is already done for parent group (like user)
	match group_key_rotation_model::get_parent_group_and_public_key(group_id.clone(), key_id.clone()).await? {
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
				})?;

			//save the keys for the parent
			group_key_rotation_model::save_user_eph_keys(group_id.clone(), key_id, user_keys).await?;
		},
		//no parent group found or the parent group is already done (e.g. was rotation starter)
		None => {},
	}

	//save the user action
	user_service::save_user_action(
		app_id,
		group_id, //use the group id as user id
		user_service::UserAction::KeyRotation,
		total_len as i64,
	)
	.await?;

	Ok(())
}

fn encrypt(eph_key: &KeyRotationWorkerKey, users: Vec<UserGroupPublicKeyData>) -> Vec<UserEphKeyOut>
{
	let mut encrypted_keys: Vec<UserEphKeyOut> = Vec::with_capacity(users.len());

	for user in users {
		//encrypt with sdk -> import public key data from string

		let encrypted_ephemeral_key = match sentc_crypto::util::server::encrypt_ephemeral_group_key_with_public_key(
			user.public_key.as_str(),
			user.public_key_alg.as_str(),
			eph_key.encrypted_ephemeral_key.as_str(),
		) {
			Ok(k) => k,
			Err(e) => {
				//don't interrupt when err, save it in the db and let the client handle it
				let ob = UserEphKeyOut {
					user_id: user.user_id,
					encrypted_ephemeral_key: "".to_string(),
					encrypted_eph_key_key_id: user.public_key_id,
					rotation_err: Some(e.into()),
				};

				encrypted_keys.push(ob);

				continue;
			},
		};

		let ob = UserEphKeyOut {
			user_id: user.user_id,
			encrypted_ephemeral_key,
			encrypted_eph_key_key_id: user.public_key_id,
			rotation_err: None,
		};
		encrypted_keys.push(ob);
	}

	encrypted_keys
}

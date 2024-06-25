use std::sync::Arc;

use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto::traverse_keys;
use sentc_crypto::util::server::encrypt_ephemeral_group_key_with_public_key;
use sentc_crypto_common::{AppId, GroupId, SymKeyId};

use crate::group::group_entities::{KeyRotationWorkerKey, UserEphKeyOut, UserGroupPublicKeyData};
use crate::group::group_key_rotation::group_key_rotation_model;
use crate::user::user_service;
use crate::util::api_res::ApiErrorCodes;

enum LoopType
{
	User,
	GroupAsMember,
}

pub async fn start(app_id: AppId, group_id: GroupId, key_id: SymKeyId, user_group: Option<String>) -> AppRes<()>
{
	let key = group_key_rotation_model::get_new_key(&group_id, &key_id).await?;

	let key_arc = Arc::new(key);

	//get all for the user
	let mut total_len = loop_user(&group_id, &key_id, key_arc.clone(), LoopType::User, &user_group).await?;

	//don't call parent key rotation or group as member key rotation for user groups
	if user_group.is_none() {
		total_len += loop_user(
			&group_id,
			&key_id,
			key_arc.clone(),
			LoopType::GroupAsMember,
			&user_group,
		)
		.await?;
	}

	//key rotation for parent group. check first if this is already done for parent group (like user)
	if let Some(item) = group_key_rotation_model::get_parent_group_and_public_key(&group_id, &key_id).await? {
		let user_keys = tokio::task::spawn_blocking(move || encrypt(&key_arc, vec![item]))
			.await
			.map_err(|e| {
				ServerCoreError::new_msg_and_debug(
					400,
					ApiErrorCodes::GroupKeyRotationThread,
					"Error in user key rotation",
					Some(format!("error in user key rotation: {}", e)),
				)
			})?;

		//save the keys for the parent
		group_key_rotation_model::save_user_eph_keys(&group_id, &key_id, user_keys).await?;
	}

	//delete the eph key which was encrypted by the last group key to avoid leaking the key
	group_key_rotation_model::delete_eph_key(&group_id, &key_id).await?;

	//save the user action
	user_service::save_user_action(
		&app_id,
		&group_id, //use the group id as user id
		user_service::UserAction::KeyRotation,
		total_len as i64,
	)
	.await?;

	Ok(())
}

async fn loop_user(
	group_id: &str,
	key_id: &str,
	key_arc: Arc<KeyRotationWorkerKey>,
	loop_type: LoopType,
	user_group: &Option<String>,
) -> AppRes<usize>
{
	let mut last_time_fetched = 0;
	let mut last_user_id = "".to_string();

	let mut total_len = 0_usize;

	loop {
		let key_cap = key_arc.clone();

		let users = match (&loop_type, user_group) {
			(LoopType::GroupAsMember, None) => {
				//get the data for the group as member
				group_key_rotation_model::get_group_as_member_public_key(group_id, key_id, last_time_fetched, &last_user_id).await?
			},
			(LoopType::User, Some(u_id)) => {
				//for a user group key rotation use the device id as user id and as public key id
				group_key_rotation_model::get_device_keys(u_id, key_id, last_time_fetched, &last_user_id).await?
			},
			(LoopType::User, None) => {
				//normal fallback to fetch all users for a group
				group_key_rotation_model::get_user_and_public_key(group_id, key_id, last_time_fetched, &last_user_id).await?
			},
			(LoopType::GroupAsMember, Some(_)) => {
				//Don't call the loop again with user group because user group won't get any group as member
				return Ok(0);
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
				ServerCoreError::new_msg_and_debug(
					400,
					ApiErrorCodes::GroupKeyRotationThread,
					"Error in user key rotation",
					Some(format!("error in user key rotation: {}", e)),
				)
			})?;

		//save the keys for the user
		group_key_rotation_model::save_user_eph_keys(group_id, key_id, user_keys).await?;

		if len < 100 {
			//when there were less than 50 users in this fetch
			break;
		}
	}

	Ok(total_len)
}

fn encrypt(eph_key: &KeyRotationWorkerKey, users: Vec<UserGroupPublicKeyData>) -> Vec<UserEphKeyOut>
{
	//TODO add new encrypt fn with request that does a req to encrypt all user keys when the app owner added an endpoint

	let mut encrypted_keys: Vec<UserEphKeyOut> = Vec::with_capacity(users.len());

	for user in users {
		//encrypt with sdk -> import public key data from string

		#[cfg(not(feature = "external_c_keys"))]
		let encrypted_ephemeral_key = traverse_keys!(
			encrypt_ephemeral_group_key_with_public_key,
			(
				&user.public_key,
				&user.public_key_alg,
				&eph_key.encrypted_ephemeral_key
			),
			[sentc_crypto_std_keys::util::SecretKey]
		);

		#[cfg(feature = "external_c_keys")]
		let encrypted_ephemeral_key = traverse_keys!(
			encrypt_ephemeral_group_key_with_public_key,
			(
				&user.public_key,
				&user.public_key_alg,
				&eph_key.encrypted_ephemeral_key
			),
			[sentc_crypto_std_keys::util::SecretKey, sentc_crypto_fips_keys::util::SecretKey]
		);

		let encrypted_ephemeral_key = match encrypted_ephemeral_key {
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

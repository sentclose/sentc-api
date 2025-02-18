use rustgram_server_util::res::AppRes;
use sentc_crypto_common::group::{KeyRotationData, KeyRotationStartServerOutput};
use sentc_crypto_common::{AppId, GroupId, UserId};
use server_key_store::KeyStorage;

pub use self::group_key_rotation_model::{done_key_rotation_for_user, get_keys_for_key_update as get_keys_for_update};
use crate::group::group_key_rotation::group_key_rotation_model;
use crate::group::group_key_rotation_worker;

pub async fn start_key_rotation(
	app_id: impl Into<AppId>,
	group_id: impl Into<GroupId>,
	starter_id: impl Into<UserId>,
	input: KeyRotationData,
	user_group: Option<String>,
) -> AppRes<KeyRotationStartServerOutput>
{
	let app_id = app_id.into();
	let group_id = group_id.into();

	let (encrypted_sign_key_for_model, encrypted_sign_key_for_storage, verify_key_for_model, verify_key_for_storage) =
		if let (Some(encrypted_sign_key), Some(verify_key)) = (input.encrypted_sign_key, input.verify_key) {
			(
				Some("extern".to_string()),
				Some(encrypted_sign_key),
				Some("extern".to_string()),
				Some(verify_key),
			)
		} else {
			(None, None, None, None)
		};

	let key_data = KeyRotationData {
		encrypted_group_key_by_user: input.encrypted_group_key_by_user,
		group_key_alg: input.group_key_alg,
		encrypted_group_key_alg: input.encrypted_group_key_alg,
		encrypted_private_group_key: "extern".to_string(),
		public_group_key: "extern".to_string(),
		keypair_encrypt_alg: input.keypair_encrypt_alg,
		encrypted_group_key_by_ephemeral: input.encrypted_group_key_by_ephemeral,
		ephemeral_alg: input.ephemeral_alg,
		encrypted_ephemeral_key: input.encrypted_ephemeral_key,
		previous_group_key_id: input.previous_group_key_id,
		invoker_public_key_id: input.invoker_public_key_id,
		signed_by_user_id: input.signed_by_user_id,
		signed_by_user_sign_key_id: input.signed_by_user_sign_key_id,
		group_key_sig: input.group_key_sig,
		encrypted_sign_key: encrypted_sign_key_for_model,
		verify_key: verify_key_for_model,
		keypair_sign_alg: input.keypair_sign_alg,
		public_key_sig: input.public_key_sig,
	};

	let key_id = group_key_rotation_model::start_key_rotation(&app_id, &group_id, starter_id, key_data).await?;

	let key_vec = if let (Some(sign_key), Some(verify_key)) = (encrypted_sign_key_for_storage, verify_key_for_storage) {
		vec![
			KeyStorage {
				key: input.public_group_key,
				id: format!("pk_{key_id}"),
			},
			KeyStorage {
				key: input.encrypted_private_group_key,
				id: format!("sk_{key_id}"),
			},
			KeyStorage {
				key: verify_key,
				id: format!("vk_{key_id}"),
			},
			KeyStorage {
				key: sign_key,
				id: format!("sign_k_{key_id}"),
			},
		]
	} else {
		vec![
			KeyStorage {
				key: input.public_group_key,
				id: format!("pk_{key_id}"),
			},
			KeyStorage {
				key: input.encrypted_private_group_key,
				id: format!("sk_{key_id}"),
			},
		]
	};

	server_key_store::upload_key(key_vec).await?;

	//don't wait for the response
	tokio::task::spawn(group_key_rotation_worker::start(
		app_id,
		group_id.clone(),
		key_id.clone(),
		user_group,
	));

	let out = KeyRotationStartServerOutput {
		key_id,
		group_id,
	};

	Ok(out)
}

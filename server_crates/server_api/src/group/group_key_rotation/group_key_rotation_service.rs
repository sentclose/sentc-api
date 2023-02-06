use std::future::Future;

use sentc_crypto_common::group::{DoneKeyRotationData, KeyRotationData, KeyRotationStartServerOutput};
use server_core::str_t;

use crate::group::group_entities::GroupKeyUpdate;
use crate::group::group_key_rotation::group_key_rotation_model;
use crate::group::group_key_rotation_worker;
use crate::util::api_res::AppRes;

pub async fn start_key_rotation(
	app_id: &str,
	group_id: &str,
	starter_id: &str,
	input: KeyRotationData,
	user_group: Option<String>,
) -> AppRes<KeyRotationStartServerOutput>
{
	let key_id = group_key_rotation_model::start_key_rotation(app_id, group_id, starter_id, input).await?;

	//dont wait for the response
	tokio::task::spawn(group_key_rotation_worker::start(
		app_id.to_string(),
		group_id.to_string(),
		key_id.clone(),
		user_group,
	));

	let out = KeyRotationStartServerOutput {
		key_id,
		group_id: group_id.to_string(),
	};

	Ok(out)
}

pub fn get_keys_for_update<'a>(
	app_id: str_t!('a),
	group_id: str_t!('a),
	user_id: str_t!('a),
) -> impl Future<Output = AppRes<Vec<GroupKeyUpdate>>> + 'a
{
	group_key_rotation_model::get_keys_for_key_update(app_id, group_id, user_id)
}

pub fn done_key_rotation_for_user<'a>(
	group_id: str_t!('a),
	user_id: str_t!('a),
	key_id: str_t!('a),
	input: DoneKeyRotationData,
) -> impl Future<Output = AppRes<()>> + 'a
{
	group_key_rotation_model::done_key_rotation_for_user(group_id, user_id, key_id, input)
}

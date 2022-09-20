use std::future::Future;

use sentc_crypto::sdk_common::AppId;
use sentc_crypto_common::group::{DoneKeyRotationData, KeyRotationData, KeyRotationStartServerOutput};
use sentc_crypto_common::{GroupId, SymKeyId, UserId};

use crate::group::group_entities::GroupKeyUpdate;
use crate::group::group_key_rotation::group_key_rotation_model;
use crate::group::group_key_rotation_worker;
use crate::util::api_res::AppRes;

pub async fn start_key_rotation(
	app_id: AppId,
	group_id: GroupId,
	starter_id: UserId,
	input: KeyRotationData,
	user_group: Option<String>,
) -> AppRes<KeyRotationStartServerOutput>
{
	let key_id = group_key_rotation_model::start_key_rotation(app_id.clone(), group_id.clone(), starter_id, input).await?;

	//dont wait for the response
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

pub fn get_keys_for_update(app_id: AppId, group_id: GroupId, user_id: UserId) -> impl Future<Output = AppRes<Vec<GroupKeyUpdate>>>
{
	group_key_rotation_model::get_keys_for_key_update(app_id, group_id, user_id)
}

pub fn done_key_rotation_for_user(
	group_id: GroupId,
	user_id: UserId,
	key_id: SymKeyId,
	input: DoneKeyRotationData,
) -> impl Future<Output = AppRes<()>>
{
	group_key_rotation_model::done_key_rotation_for_user(group_id, user_id, key_id, input)
}

use rustgram_server_util::res::AppRes;
use sentc_crypto_common::group::{KeyRotationData, KeyRotationStartServerOutput};
use sentc_crypto_common::{AppId, GroupId, UserId};

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

	let key_id = group_key_rotation_model::start_key_rotation(&app_id, &group_id, starter_id, input).await?;

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

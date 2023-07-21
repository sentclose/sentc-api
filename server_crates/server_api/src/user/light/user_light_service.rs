use rustgram_server_util::cache;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::user::{RegisterServerOutput, UserDeviceDoneRegisterInputLight, UserDeviceLightRegisterInput, UserDeviceRegisterOutput};
use sentc_crypto_common::{AppId, GroupId, UserId};

use crate::group::{group_service, group_user_service, GROUP_TYPE_USER};
use crate::sentc_app_utils::hash_token_to_string;
use crate::sentc_group_entities::{InternalGroupData, InternalGroupDataComplete, InternalUserGroupData};
use crate::sentc_group_user_service::NewUserType;
use crate::sentc_user_service::create_refresh_token;
use crate::user::light::user_light_model;
use crate::user::user_model;
use crate::util::api_res::ApiErrorCodes;
use crate::util::get_user_in_app_key;

pub async fn register_light(app_id: impl Into<AppId>, input: UserDeviceLightRegisterInput, user: bool) -> AppRes<RegisterServerOutput>
{
	let app_id = app_id.into();

	let identifier = hash_token_to_string(input.device_identifier.as_bytes())?;

	let (user_id, device_id) = user_light_model::register_light(&app_id, identifier, input.master_key, input.derived).await?;

	//delete the user in app check cache from the jwt mw
	//it can happened that a user id was used before which doesn't exists yet
	let cache_key = get_user_in_app_key(&app_id, &user_id);
	cache::delete(&cache_key).await?;

	if user {
		//creat the user group for the user devices
		let group_id = group_service::create_group_light(&app_id, &user_id, GROUP_TYPE_USER, None, None, None, false).await?;

		user_model::register_update_user_group_id(app_id, &user_id, group_id).await?;
	}

	Ok(RegisterServerOutput {
		user_id,
		device_id,
		device_identifier: input.device_identifier,
	})
}

pub async fn prepare_register_device_light(app_id: impl Into<AppId>, input: UserDeviceLightRegisterInput) -> AppRes<UserDeviceRegisterOutput>
{
	let app_id = app_id.into();
	let check = user_model::check_user_exists(&app_id, &input.device_identifier).await?;

	if check {
		//check true == user exists
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::UserExists,
			"Identifier already exists",
		));
	}

	let public_key_string = input.derived.public_key.to_string();
	let keypair_encrypt_alg = input.derived.keypair_encrypt_alg.to_string();

	let identifier = hash_token_to_string(input.device_identifier.as_bytes())?;

	let token = create_refresh_token()?;

	let device_id = user_light_model::register_device_light(app_id, identifier, input.master_key, input.derived, &token).await?;

	Ok(UserDeviceRegisterOutput {
		device_id,
		token,
		device_identifier: input.device_identifier,
		public_key_string,
		keypair_encrypt_alg,
	})
}

pub async fn done_register_device_light(
	app_id: impl Into<AppId>,
	user_id: impl Into<UserId>,
	user_group_id: impl Into<GroupId>,
	input: UserDeviceDoneRegisterInputLight,
) -> AppRes<()>
{
	let app_id = app_id.into();

	let device_id = user_model::get_done_register_device(&app_id, input.token).await?;

	group_user_service::invite_auto_light(
		&InternalGroupDataComplete {
			group_data: InternalGroupData {
				app_id: app_id.clone(),
				id: user_group_id.into(),
				time: 0,
				parent: None,
				invite: 1, //must be 1 to accept the device invite
				is_connected_group: false,
			},
			user_data: InternalUserGroupData {
				user_id: "".to_string(),
				real_user_id: "".to_string(),
				joined_time: 0,
				rank: 0, //Rank must be 0
				get_values_from_parent: None,
				get_values_from_group_as_member: None,
			},
		},
		input.user_group,
		&device_id, //invite the new device
		NewUserType::Normal,
	)
	.await?;

	user_model::done_register_device(app_id, user_id, device_id).await?;

	Ok(())
}
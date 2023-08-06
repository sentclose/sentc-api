use rustgram_server_util::cache;
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::user::{
	RegisterServerOutput,
	UserDeviceDoneRegisterInputLight,
	UserDeviceRegisterInput,
	VerifyLoginInput,
	VerifyLoginLightOutput,
};
use sentc_crypto_common::{AppId, GroupId, UserId};

use crate::group::{group_service, group_user_service, GROUP_TYPE_USER};
use crate::sentc_app_entities::AppData;
use crate::sentc_group_user_service::NewUserType;
use crate::sentc_user_entities::{LoginForcedLightOutput, VerifyLoginEntity};
use crate::sentc_user_service::internal_group_data;
use crate::user::auth::auth_service;
use crate::user::light::user_light_model;
use crate::user::user_model;
use crate::util::{get_user_in_app_key, hash_token_to_string};

pub async fn register_light(app_id: impl Into<AppId>, input: UserDeviceRegisterInput, user: bool) -> AppRes<RegisterServerOutput>
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
		let group_id = group_service::create_group_light(&app_id, &device_id, GROUP_TYPE_USER, None, None, None, false).await?;

		user_model::register_update_user_group_id(app_id, &user_id, group_id).await?;
	}

	Ok(RegisterServerOutput {
		user_id,
		device_id,
		device_identifier: input.device_identifier,
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
		&internal_group_data(&app_id, user_group_id, 0),
		input.user_group,
		&device_id, //invite the new device
		NewUserType::Normal,
	)
	.await?;

	user_model::done_register_device(app_id, user_id, device_id).await?;

	Ok(())
}

pub async fn verify_login_light(app_data: &AppData, done_login: VerifyLoginInput) -> AppRes<(VerifyLoginLightOutput, VerifyLoginEntity)>
{
	let (data, jwt, refresh_token) = auth_service::verify_login_internally(app_data, done_login).await?;

	Ok((
		VerifyLoginLightOutput {
			jwt,
			refresh_token,
		},
		data,
	))
}

pub async fn verify_login_light_forced(app_data: &AppData, identifier: &str) -> AppRes<LoginForcedLightOutput>
{
	let (data, jwt, refresh_token) = auth_service::verify_login_forced_internally(app_data, identifier).await?;

	Ok(LoginForcedLightOutput {
		device_keys: data,
		verify: VerifyLoginLightOutput {
			jwt,
			refresh_token,
		},
	})
}

pub async fn reset_password_light(app_id: impl Into<AppId>, input: UserDeviceRegisterInput) -> AppRes<()>
{
	let identifier = hash_token_to_string(input.device_identifier.as_bytes())?;

	user_light_model::reset_password_light(app_id, identifier, input.master_key, input.derived).await?;

	Ok(())
}

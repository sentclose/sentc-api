use rustgram::Request;
use sentc_crypto_common::group::{
	DoneKeyRotationData,
	GroupAcceptJoinReqServerOutput,
	GroupKeysForNewMember,
	KeyRotationData,
	KeyRotationStartServerOutput,
};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::user::{
	ChangePasswordData,
	DoneLoginLightServerOutput,
	DoneLoginServerInput,
	JwtRefreshInput,
	PrepareLoginSaltServerOutput,
	PrepareLoginServerInput,
	RegisterData,
	RegisterServerOutput,
	ResetPasswordData,
	UserDeviceDoneRegisterInput,
	UserDeviceRegisterInput,
	UserDeviceRegisterOutput,
	UserIdentifierAvailableServerInput,
	UserIdentifierAvailableServerOutput,
	UserUpdateServerInput,
};
use server_core::input_helper::{bytes_to_json, get_raw_body};
use server_core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};

use crate::customer_app::app_util::{check_endpoint_with_app_options, get_app_data_from_req, Endpoint};
use crate::group::group_entities::{GroupKeyUpdate, GroupUserKeys};
use crate::group::{group_key_rotation_service, group_user_service};
use crate::user::jwt::get_jwt_data_from_param;
use crate::user::user_entities::{DoneLoginServerOutput, UserDeviceList, UserInitEntity, UserPublicKeyDataEntity, UserVerifyKeyDataEntity};
use crate::user::user_service::UserAction;
use crate::user::{user_model, user_service};
use crate::util::api_res::{echo, echo_success, ApiErrorCodes, HttpErr, JRes};

pub(crate) async fn exists(mut req: Request) -> JRes<UserIdentifierAvailableServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let data: UserIdentifierAvailableServerInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserExists)?;

	let out = user_service::exists(app_data, data).await?;

	echo(out)
}

pub(crate) async fn register(mut req: Request) -> JRes<RegisterServerOutput>
{
	//load the register input from the req body
	let body = get_raw_body(&mut req).await?;
	let register_input: RegisterData = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserRegister)?;

	let out = user_service::register(app_data.app_data.app_id.clone(), register_input).await?;

	echo(out)
}

pub(crate) async fn prepare_register_device(mut req: Request) -> JRes<UserDeviceRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: UserDeviceRegisterInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserDeviceRegister)?;

	let out = user_service::prepare_register_device(app_data.app_data.app_id.clone(), input).await?;

	echo(out)
}

pub(crate) async fn done_register_device(mut req: Request) -> JRes<GroupAcceptJoinReqServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: UserDeviceDoneRegisterInput = bytes_to_json(&body)?;
	let app = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	check_endpoint_with_app_options(app, Endpoint::UserDeviceRegister)?;

	let session_id = user_service::done_register_device(
		app.app_data.app_id.clone(),
		user.id.clone(),
		user.group_id.clone(),
		input,
	)
	.await?;

	let out = GroupAcceptJoinReqServerOutput {
		session_id,
		message: "This device was added to the account.".to_string(),
	};

	echo(out)
}

pub(crate) async fn device_key_upload(mut req: Request) -> JRes<ServerSuccessOutput>
{
	//the same as group user key but with the user device. the user id in the key session is the device id
	let body = get_raw_body(&mut req).await?;
	let input: Vec<GroupKeysForNewMember> = bytes_to_json(&body)?;

	let user = get_jwt_data_from_param(&req)?;
	let key_session_id = get_name_param_from_req(&req, "key_session_id")?;

	group_user_service::insert_user_keys_via_session(
		user.group_id.clone(),
		key_session_id.to_string(),
		group_user_service::InsertNewUserType::Join,
		input,
	)
	.await?;

	echo_success()
}

//__________________________________________________________________________________________________

pub(crate) async fn prepare_login(mut req: Request) -> JRes<PrepareLoginSaltServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user_identifier: PrepareLoginServerInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPrepLogin)?;

	let out = user_service::prepare_login(app_data, user_identifier).await?;

	echo(out)
}

pub(crate) async fn done_login(mut req: Request) -> JRes<DoneLoginServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let done_login: DoneLoginServerInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserDoneLogin)?;

	let out = user_service::done_login(app_data, done_login).await?;

	//save the action, only in controller not service because this just not belongs to other controller
	user_model::save_user_action(
		app_data.app_data.app_id.clone(),
		out.device_keys.user_id.clone(),
		UserAction::Login,
		1,
	)
	.await?;

	echo(out)
}

pub(crate) async fn get_user_keys(req: Request) -> JRes<Vec<GroupUserKeys>>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::UserDoneLogin)?;

	let user = get_jwt_data_from_param(&req)?;

	let params = get_params(&req)?;
	let last_k_id = get_name_param_from_params(params, "last_k_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let user_keys = user_service::get_user_keys(
		user,
		app.app_data.app_id.clone(),
		last_fetched_time,
		last_k_id.to_string(),
	)
	.await?;

	echo(user_keys)
}

pub(crate) async fn get_user_key(req: Request) -> JRes<GroupUserKeys>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::UserDoneLogin)?;

	let user = get_jwt_data_from_param(&req)?;
	let key_id = get_name_param_from_req(&req, "key_id")?;

	let user_key = user_service::get_user_key(user, app.app_data.app_id.clone(), key_id.to_string()).await?;

	echo(user_key)
}

//__________________________________________________________________________________________________

pub(crate) async fn get_public_key_by_id(req: Request) -> JRes<UserPublicKeyDataEntity>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPublicData)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;
	let public_key_id = get_name_param_from_req(&req, "key_id")?;

	let out = user_model::get_public_key_by_id(
		app_data.app_data.app_id.clone(),
		user_id.to_string(),
		public_key_id.to_string(),
	)
	.await?;

	echo(out)
}

pub(crate) async fn get_public_key_data(req: Request) -> JRes<UserPublicKeyDataEntity>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPublicData)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;

	let data = user_model::get_public_key_data(app_data.app_data.app_id.clone(), user_id.to_string()).await?;

	echo(data)
}

pub(crate) async fn get_verify_key_by_id(req: Request) -> JRes<UserVerifyKeyDataEntity>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPublicData)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;
	let verify_key_id = get_name_param_from_req(&req, "key_id")?;

	let out = user_model::get_verify_key_by_id(
		app_data.app_data.app_id.clone(),
		user_id.to_string(),
		verify_key_id.to_string(),
	)
	.await?;

	echo(out)
}

//__________________________________________________________________________________________________
// user fn with jwt

pub(crate) async fn init_user(mut req: Request) -> JRes<UserInitEntity>
{
	let body = get_raw_body(&mut req).await?;
	let input: JwtRefreshInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserRefreshJwt)?;

	//this can be an expired jwt, but the app id must be valid
	let user = get_jwt_data_from_param(&req)?;

	let out = user_service::init_user(app_data, user.device_id.clone(), input).await?;

	user_model::save_user_action(app_data.app_data.app_id.clone(), user.id.clone(), UserAction::Init, 1).await?;

	echo(out)
}

pub(crate) async fn refresh_jwt(mut req: Request) -> JRes<DoneLoginLightServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: JwtRefreshInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserRefreshJwt)?;

	//this can be an expired jwt, but the app id must be valid
	//to get the old token in the client when init the user client -> save the old jwt in the client like the keys
	let user = get_jwt_data_from_param(&req)?;

	let out = user_service::refresh_jwt(app_data, user.device_id.clone(), input).await?;

	user_model::save_user_action(
		app_data.app_data.app_id.clone(),
		out.user_id.clone(),
		UserAction::Refresh,
		1,
	)
	.await?;

	echo(out)
}

pub(crate) async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::UserDelete)?;

	let user = get_jwt_data_from_param(&req)?;

	user_service::delete(user, app.app_data.app_id.clone()).await?;

	user_model::save_user_action(
		app.app_data.app_id.to_string(),
		user.id.clone(),
		UserAction::Delete,
		1,
	)
	.await?;

	echo_success()
}

pub(crate) async fn delete_device(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::UserDeviceDelete)?;

	let user = get_jwt_data_from_param(&req)?;
	let device_id = get_name_param_from_req(&req, "device_id")?;

	user_service::delete_device(user, app.app_data.app_id.clone(), device_id.to_string()).await?;

	echo_success()
}

pub(crate) async fn get_devices(req: Request) -> JRes<Vec<UserDeviceList>>
{
	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::UserDeviceList)?;

	let user = get_jwt_data_from_param(&req)?;

	let params = get_params(&req)?;
	let last_id = get_name_param_from_params(params, "last_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let out = user_service::get_devices(
		app.app_data.app_id.clone(),
		user.id.clone(),
		last_fetched_time,
		last_id.to_string(),
	)
	.await?;

	echo(out)
}

pub(crate) async fn update(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let update_input: UserUpdateServerInput = bytes_to_json(&body)?;

	let app = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	check_endpoint_with_app_options(app, Endpoint::UserUpdate)?;

	user_service::update(user, app.app_data.app_id.clone(), update_input).await?;

	echo_success()
}

pub(crate) async fn change_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user = get_jwt_data_from_param(&req)?;
	let app_data = get_app_data_from_req(&req)?;

	let input: ChangePasswordData = bytes_to_json(&body)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserChangePassword)?;

	user_service::change_password(user, app_data.app_data.app_id.to_string(), input).await?;

	user_model::save_user_action(
		app_data.app_data.app_id.clone(),
		user.id.clone(),
		UserAction::ChangePassword,
		1,
	)
	.await?;

	echo_success()
}

pub(crate) async fn reset_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user = get_jwt_data_from_param(&req)?; //non fresh jwt here
	let app_data = get_app_data_from_req(&req)?;
	let input: ResetPasswordData = bytes_to_json(&body)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserResetPassword)?;

	user_service::reset_password(user.id.clone(), user.device_id.clone(), input).await?;

	user_model::save_user_action(
		app_data.app_data.app_id.clone(),
		user.id.clone(),
		UserAction::ResetPassword,
		1,
	)
	.await?;

	echo_success()
}

//__________________________________________________________________________________________________

pub(crate) async fn user_group_key_rotation(mut req: Request) -> JRes<KeyRotationStartServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user = get_jwt_data_from_param(&req)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserKeyRotation)?;

	let input: KeyRotationData = bytes_to_json(&body)?;

	let group_id = user_model::prepare_user_key_rotation(app_data.app_data.app_id.clone(), user.id.clone()).await?;

	let group_id = match group_id {
		Some(id) => id.0,
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::UserNotFound,
				"User not found".to_string(),
				None,
			))
		},
	};

	let out = group_key_rotation_service::start_key_rotation(
		app_data.app_data.app_id.clone(),
		group_id,
		user.device_id.clone(),
		input,
		Some(user.id.clone()),
	)
	.await?;

	echo(out)
}

pub(crate) async fn get_user_group_keys_for_update(req: Request) -> JRes<Vec<GroupKeyUpdate>>
{
	let user = get_jwt_data_from_param(&req)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserKeyRotation)?;

	let update = group_key_rotation_service::get_keys_for_update(
		app_data.app_data.app_id.clone(),
		user.group_id.clone(),
		user.device_id.clone(),
	)
	.await?;

	echo(update)
}

pub(crate) async fn done_key_rotation_for_device(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	let user = get_jwt_data_from_param(&req)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserKeyRotation)?;

	let key_id = get_name_param_from_req(&req, "key_id")?;

	let input: DoneKeyRotationData = bytes_to_json(&body)?;

	group_key_rotation_service::done_key_rotation_for_user(
		user.group_id.clone(),
		user.device_id.clone(),
		key_id.to_string(),
		input,
	)
	.await?;

	echo_success()
}

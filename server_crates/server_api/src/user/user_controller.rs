use rustgram::Request;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, JRes, ServerSuccessOutput};
use rustgram_server_util::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params, get_time_from_url_param};
use sentc_crypto_common::group::{
	DoneKeyRotationData,
	GroupAcceptJoinReqServerOutput,
	GroupKeysForNewMember,
	KeyRotationData,
	KeyRotationStartServerOutput,
};
use sentc_crypto_common::user::{
	ChangePasswordData,
	DoneLoginLightServerOutput,
	DoneLoginServerInput,
	JwtRefreshInput,
	LoginForcedInput,
	OtpInput,
	OtpRecoveryKeysOutput,
	OtpRegister,
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
	UserJwtInfo,
	UserUpdateServerInput,
	VerifyLoginInput,
};

use crate::customer_app::app_util::{check_endpoint_with_app_options, get_app_data_from_req, Endpoint};
use crate::group::group_entities::{GroupKeyUpdate, GroupUserKeys};
use crate::group::{group_key_rotation_service, group_user_service};
use crate::sentc_app_utils::check_endpoint_with_req;
use crate::sentc_user_entities::{DoneLoginServerOutput, DoneLoginServerReturn, LoginForcedOutput, VerifyLoginOutput};
use crate::user::auth::auth_service;
use crate::user::jwt::get_jwt_data_from_param;
use crate::user::user_entities::{UserDeviceList, UserInitEntity, UserPublicKeyDataEntity, UserVerifyKeyDataEntity};
use crate::user::user_service::UserAction;
use crate::user::{user_model, user_service};
use crate::util::api_res::ApiErrorCodes;

pub(crate) async fn exists(mut req: Request) -> JRes<UserIdentifierAvailableServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let data: UserIdentifierAvailableServerInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserExists)?;

	let out = user_service::exists(&app_data.app_data.app_id, data).await?;

	echo(out)
}

pub(crate) async fn register(mut req: Request) -> JRes<RegisterServerOutput>
{
	//load the register input from the req body
	let body = get_raw_body(&mut req).await?;
	let register_input: RegisterData = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserRegister)?;

	let out = user_service::register(&app_data.app_data.app_id, register_input).await?;

	echo(out)
}

pub(crate) async fn prepare_register_device(mut req: Request) -> JRes<UserDeviceRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: UserDeviceRegisterInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserDeviceRegister)?;

	let out = user_service::prepare_register_device(&app_data.app_data.app_id, input).await?;

	echo(out)
}

pub(crate) async fn done_register_device(mut req: Request) -> JRes<GroupAcceptJoinReqServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: UserDeviceDoneRegisterInput = bytes_to_json(&body)?;
	let app = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	check_endpoint_with_app_options(app, Endpoint::UserDeviceRegister)?;

	let session_id = user_service::done_register_device(&app.app_data.app_id, &user.id, &user.group_id, input).await?;

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
		&user.group_id,
		key_session_id,
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

	let out = auth_service::prepare_login(app_data, &user_identifier.user_identifier).await?;

	echo(out)
}

pub(crate) async fn done_login(mut req: Request) -> JRes<DoneLoginServerReturn>
{
	let body = get_raw_body(&mut req).await?;
	let done_login: DoneLoginServerInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserDoneLogin)?;

	let out = auth_service::done_login(app_data, done_login).await?;

	if let DoneLoginServerReturn::Direct(d) = &out {
		//save the action, only in controller not service because this just not belongs to other controller
		user_model::save_user_action(
			&app_data.app_data.app_id,
			&d.device_keys.user_id,
			UserAction::Login,
			1,
		)
		.await?;
	}

	echo(out)
}

pub(crate) async fn validate_mfa(mut req: Request) -> JRes<DoneLoginServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: OtpInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app_data, Endpoint::UserDoneLogin)?;

	let out = auth_service::validate_mfa(app_data, input).await?;

	//2fa do there the user action
	user_model::save_user_action(
		&app_data.app_data.app_id,
		&out.device_keys.user_id,
		UserAction::Login,
		1,
	)
	.await?;

	echo(out)
}

pub(crate) async fn validate_recovery_otp(mut req: Request) -> JRes<DoneLoginServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: OtpInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app_data, Endpoint::UserDoneLogin)?;

	let out = auth_service::validate_recovery_otp(app_data, input).await?;

	//2fa do there the user action
	user_model::save_user_action(
		&app_data.app_data.app_id,
		&out.device_keys.user_id,
		UserAction::Login,
		1,
	)
	.await?;

	echo(out)
}

pub(crate) async fn verify_login(mut req: Request) -> JRes<VerifyLoginOutput>
{
	//return here the jwt and the refresh token when he challenge was correct.

	let body = get_raw_body(&mut req).await?;
	let done_login: VerifyLoginInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserDoneLogin)?;

	let out = user_service::verify_login(app_data, done_login).await?;

	echo(out)
}

pub(crate) async fn verify_login_forced(mut req: Request) -> JRes<LoginForcedOutput>
{
	//Fn to skip the login process and just return the user data

	let body = get_raw_body(&mut req).await?;
	let user_identifier: LoginForcedInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app_data, Endpoint::ForceServer)?;

	let out = user_service::verify_login_forced(app_data, &user_identifier.user_identifier).await?;

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
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let user_keys = user_service::get_user_keys(user, &app.app_data.app_id, last_fetched_time, last_k_id).await?;

	echo(user_keys)
}

pub(crate) async fn get_user_key(req: Request) -> JRes<GroupUserKeys>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::UserDoneLogin)?;

	let user = get_jwt_data_from_param(&req)?;
	let key_id = get_name_param_from_req(&req, "key_id")?;

	let user_key = user_service::get_user_key(user, &app.app_data.app_id, key_id).await?;

	echo(user_key)
}

//__________________________________________________________________________________________________

pub async fn get_public_key_by_id(req: Request) -> JRes<UserPublicKeyDataEntity>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPublicData)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;
	let public_key_id = get_name_param_from_req(&req, "key_id")?;

	let out = user_service::get_public_key_by_id(&app_data.app_data.app_id, user_id, public_key_id).await?;

	echo(out)
}

pub async fn get_public_key_data(req: Request) -> JRes<UserPublicKeyDataEntity>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPublicData)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;

	let data = user_service::get_public_key_data(&app_data.app_data.app_id, user_id).await?;

	echo(data)
}

pub async fn get_verify_key_by_id(req: Request) -> JRes<UserVerifyKeyDataEntity>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPublicData)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;
	let verify_key_id = get_name_param_from_req(&req, "key_id")?;

	let out = user_service::get_verify_key_by_id(&app_data.app_data.app_id, user_id, verify_key_id).await?;

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

	let out = user_service::init_user(app_data, &user.device_id, input).await?;

	user_model::save_user_action(&app_data.app_data.app_id, &user.id, UserAction::Init, 1).await?;

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

	let out = user_service::refresh_jwt(app_data, &user.device_id, input).await?;

	user_model::save_user_action(&app_data.app_data.app_id, &out.user_id, UserAction::Refresh, 1).await?;

	echo(out)
}

pub(crate) async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::UserDelete)?;

	let user = get_jwt_data_from_param(&req)?;

	user_service::delete(user, &app.app_data.app_id).await?;

	user_model::save_user_action(&app.app_data.app_id, &user.id, UserAction::Delete, 1).await?;

	echo_success()
}

pub(crate) async fn delete_device(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::UserDeviceDelete)?;

	let user = get_jwt_data_from_param(&req)?;
	let device_id = get_name_param_from_req(&req, "device_id")?;

	user_service::delete_device(user, &app.app_data.app_id, device_id).await?;

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
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let out = user_service::get_devices(&app.app_data.app_id, &user.id, last_fetched_time, last_id).await?;

	echo(out)
}

pub(crate) async fn update(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let update_input: UserUpdateServerInput = bytes_to_json(&body)?;

	let app = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	check_endpoint_with_app_options(app, Endpoint::UserUpdate)?;

	user_service::update(user, &app.app_data.app_id, update_input).await?;

	echo_success()
}

pub(crate) async fn change_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user = get_jwt_data_from_param(&req)?;
	let app_data = get_app_data_from_req(&req)?;

	let input: ChangePasswordData = bytes_to_json(&body)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserChangePassword)?;

	user_service::change_password(user, &app_data.app_data.app_id, input).await?;

	user_model::save_user_action(&app_data.app_data.app_id, &user.id, UserAction::ChangePassword, 1).await?;

	echo_success()
}

pub(crate) async fn reset_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user = get_jwt_data_from_param(&req)?; //non fresh jwt here
	let app_data = get_app_data_from_req(&req)?;
	let input: ResetPasswordData = bytes_to_json(&body)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserResetPassword)?;

	user_service::reset_password(&user.id, &user.device_id, input).await?;

	user_model::save_user_action(&app_data.app_data.app_id, &user.id, UserAction::ResetPassword, 1).await?;

	echo_success()
}

//__________________________________________________________________________________________________
//otp

pub(crate) async fn register_otp(req: Request) -> JRes<OtpRegister>
{
	let app_data = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app_data, Endpoint::UserRegisterOtp)?;

	let user = get_jwt_data_from_param(&req)?;

	let out = user_service::register_otp(&app_data.app_data.app_id, &user.id).await?;

	echo(out)
}

pub(crate) async fn reset_otp(req: Request) -> JRes<OtpRegister>
{
	let app_data = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app_data, Endpoint::UserResetOtp)?;

	let user = get_jwt_data_from_param(&req)?;

	let out = user_service::reset_otp(&app_data.app_data.app_id, user).await?;

	echo(out)
}

pub(crate) async fn disable_otp(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::UserDisableOtp)?;

	let user = get_jwt_data_from_param(&req)?;

	user_service::disable_otp(user).await?;

	echo_success()
}

pub(crate) async fn get_otp_recovery_keys(req: Request) -> JRes<OtpRecoveryKeysOutput>
{
	check_endpoint_with_req(&req, Endpoint::UserGetOtpRecoveryKeys)?;

	let user = get_jwt_data_from_param(&req)?;

	let out = user_service::get_otp_recovery_keys(user).await?;

	echo(out)
}

//__________________________________________________________________________________________________

pub(crate) async fn user_group_key_rotation(mut req: Request) -> JRes<KeyRotationStartServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user = get_jwt_data_from_param(&req)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserKeyRotation)?;

	let input: KeyRotationData = bytes_to_json(&body)?;

	if input.public_key_sig.is_none() || input.verify_key.is_none() || input.encrypted_sign_key.is_none() || input.keypair_sign_alg.is_none() {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::UserKeysNotFound,
			"User keys not found. Make sure to create the user group.",
		));
	}

	let out = group_key_rotation_service::start_key_rotation(
		&app_data.app_data.app_id,
		&user.group_id,
		&user.device_id,
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

	let update = group_key_rotation_service::get_keys_for_update(&app_data.app_data.app_id, &user.group_id, &user.device_id).await?;

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

	group_key_rotation_service::done_key_rotation_for_user(&user.group_id, &user.device_id, key_id, input).await?;

	echo_success()
}

//__________________________________________________________________________________________________

pub(crate) async fn get_user_data_from_jwt(req: Request) -> JRes<UserJwtInfo>
{
	check_endpoint_with_req(&req, Endpoint::ForceServer)?;
	let user = get_jwt_data_from_param(&req)?;

	echo(UserJwtInfo {
		id: user.id.clone(),
		device_id: user.device_id.clone(),
	})
}

//__________________________________________________________________________________________________

pub(crate) async fn delete_user(req: Request) -> JRes<ServerSuccessOutput>
{
	//this deletes the user from server without the jwt

	let user_id = get_name_param_from_req(&req, "user_id")?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::ForceServer)?;

	user_service::delete_from_server(user_id, &app_data.app_data.app_id).await?;

	echo_success()
}

pub mod jwt;
pub(crate) mod user_entities;
mod user_model;
pub mod user_service;

use rustgram::Request;
use sentc_crypto_common::group::GroupAcceptJoinReqServerOutput;
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

use crate::customer_app::app_util::{check_endpoint_with_app_options, check_endpoint_with_req, get_app_data_from_req, Endpoint};
use crate::group::group_entities::GroupUserKeys;
use crate::group::group_service;
use crate::user::jwt::get_jwt_data_from_param;
use crate::user::user_entities::{DoneLoginServerOutput, UserInitEntity, UserPublicData, UserPublicKeyDataEntity, UserVerifyKeyDataEntity};
use crate::user::user_model::UserAction;
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

	let out = user_service::register(app_data.app_data.app_id.to_string(), register_input).await?;

	echo(out)
}

pub(crate) async fn prepare_register_device(mut req: Request) -> JRes<UserDeviceRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: UserDeviceRegisterInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	let out = user_service::prepare_register_device(app_data.app_data.app_id.to_string(), input).await?;

	echo(out)
}

pub(crate) async fn done_register_device(mut req: Request) -> JRes<GroupAcceptJoinReqServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: UserDeviceDoneRegisterInput = bytes_to_json(&body)?;
	let user = get_jwt_data_from_param(&req)?;

	let session_id = user_service::done_register_device(
		user.sub.to_string(),
		user.id.to_string(),
		user.group_id.to_string(),
		input,
	)
	.await?;

	let out = GroupAcceptJoinReqServerOutput {
		session_id,
		message: "This device was added to the account.".to_string(),
	};

	echo(out)
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
		app_data.app_data.app_id.to_string(),
		out.device_keys.user_id.to_string(),
		UserAction::Login,
	)
	.await?;

	echo(out)
}

pub(crate) async fn get_user_keys(req: Request) -> JRes<Vec<GroupUserKeys>>
{
	check_endpoint_with_req(&req, Endpoint::UserDoneLogin)?;

	let user = get_jwt_data_from_param(&req)?;

	let params = get_params(&req)?;
	let last_k_id = get_name_param_from_params(&params, "last_k_id")?;
	let last_fetched_time = get_name_param_from_params(&params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let user_keys = group_service::get_user_group_keys(
		user.sub.to_string(),
		user.group_id.to_string(),
		user.device_id.to_string(), //call it with the device id to decrypt the keys
		last_fetched_time,
		last_k_id.to_string(),
	)
	.await?;

	echo(user_keys)
}

//__________________________________________________________________________________________________

pub(crate) async fn get(req: Request) -> JRes<UserPublicData>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPublicData)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;

	let data = user_model::get_public_data(app_data.app_data.app_id.to_string(), user_id.to_string()).await?;

	echo(data)
}

pub(crate) async fn get_public_key_by_id(req: Request) -> JRes<UserPublicKeyDataEntity>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPublicData)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;
	let public_key_id = get_name_param_from_req(&req, "key_id")?;

	let out = user_model::get_public_key_by_id(
		app_data.app_data.app_id.to_string(),
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

	let data = user_model::get_public_key_data(app_data.app_data.app_id.to_string(), user_id.to_string()).await?;

	echo(data)
}

pub(crate) async fn get_verify_key_by_id(req: Request) -> JRes<UserVerifyKeyDataEntity>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPublicData)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;
	let verify_key_id = get_name_param_from_req(&req, "key_id")?;

	let out = user_model::get_verify_key_by_id(
		app_data.app_data.app_id.to_string(),
		user_id.to_string(),
		verify_key_id.to_string(),
	)
	.await?;

	echo(out)
}

pub(crate) async fn get_verify_key_data(req: Request) -> JRes<UserVerifyKeyDataEntity>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserPublicData)?;

	let user_id = get_name_param_from_req(&req, "user_id")?;

	let data = user_model::get_verify_key_data(app_data.app_data.app_id.to_string(), user_id.to_string()).await?;

	echo(data)
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

	let out = user_service::init_user(app_data, user.device_id.to_string(), input).await?;

	user_model::save_user_action(
		app_data.app_data.app_id.to_string(),
		user.id.to_string(),
		UserAction::Init,
	)
	.await?;

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

	let out = user_service::refresh_jwt(app_data, user.device_id.to_string(), input, "user").await?;

	user_model::save_user_action(
		app_data.app_data.app_id.to_string(),
		out.user_id.to_string(),
		UserAction::Refresh,
	)
	.await?;

	echo(out)
}

pub(crate) async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::UserDelete)?;

	let user = get_jwt_data_from_param(&req)?;

	user_service::delete(user).await?;

	user_model::save_user_action(user.sub.to_string(), user.id.to_string(), UserAction::Delete).await?;

	echo_success()
}

pub(crate) async fn update(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let update_input: UserUpdateServerInput = bytes_to_json(&body)?;
	let user = get_jwt_data_from_param(&req)?;

	check_endpoint_with_req(&req, Endpoint::UserUpdate)?;

	user_service::update(user, update_input).await?;

	echo_success()
}

pub(crate) async fn change_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user = get_jwt_data_from_param(&req)?;
	let input: ChangePasswordData = bytes_to_json(&body)?;

	check_endpoint_with_req(&req, Endpoint::UserChangePassword)?;

	user_service::change_password(user, input).await?;

	user_model::save_user_action(user.sub.to_string(), user.id.to_string(), UserAction::ChangePassword).await?;

	echo_success()
}

pub(crate) async fn reset_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user = get_jwt_data_from_param(&req)?; //non fresh jwt here
	let input: ResetPasswordData = bytes_to_json(&body)?;

	check_endpoint_with_req(&req, Endpoint::UserResetPassword)?;

	user_service::reset_password(user.id.to_string(), user.device_id.to_string(), input).await?;

	user_model::save_user_action(user.sub.to_string(), user.id.to_string(), UserAction::ResetPassword).await?;

	echo_success()
}

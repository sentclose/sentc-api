use rustgram::Request;
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, JRes, ServerSuccessOutput};
use sentc_crypto_common::user::{
	LoginForcedInput,
	RegisterServerOutput,
	UserDeviceDoneRegisterInputLight,
	UserDeviceRegisterInput,
	VerifyLoginInput,
	VerifyLoginLightOutput,
};

use crate::sentc_app_utils::{check_endpoint_with_app_options, get_app_data_from_req, Endpoint};
use crate::sentc_user_jwt_service::get_jwt_data_from_param;
use crate::user::light::user_light_service;

pub(crate) async fn register_light(mut req: Request) -> JRes<RegisterServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let register_input: UserDeviceRegisterInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserRegister)?;

	let out = user_light_service::register_light(&app_data.app_data.app_id, register_input, true).await?;

	echo(out)
}

pub(crate) async fn done_register_device_light(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: UserDeviceDoneRegisterInputLight = bytes_to_json(&body)?;
	let app = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	check_endpoint_with_app_options(app, Endpoint::UserDeviceRegister)?;

	user_light_service::done_register_device_light(&app.app_data.app_id, &user.id, &user.group_id, input).await?;

	echo_success()
}

pub(crate) async fn verify_login_light(mut req: Request) -> JRes<VerifyLoginLightOutput>
{
	let body = get_raw_body(&mut req).await?;
	let done_login: VerifyLoginInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserDoneLogin)?;

	let (out, _) = user_light_service::verify_login_light(app_data, done_login).await?;

	echo(out)
}

pub(crate) async fn verify_login_light_forced(mut req: Request) -> JRes<VerifyLoginLightOutput>
{
	//Fn to skip the login process and just return the user data

	let body = get_raw_body(&mut req).await?;
	let user_identifier: LoginForcedInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app_data, Endpoint::ForceServer)?;

	let out = user_light_service::verify_login_light_forced(app_data, &user_identifier.user_identifier).await?;

	echo(out)
}

pub(crate) async fn reset_password_light(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: UserDeviceRegisterInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::ForceServer)?;

	user_light_service::reset_password_light(&app_data.app_data.app_id, input).await?;

	echo_success()
}

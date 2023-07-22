use rustgram::Request;
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, JRes, ServerSuccessOutput};
use sentc_crypto_common::user::{
	DoneLoginLightOutput,
	DoneLoginServerInput,
	RegisterServerOutput,
	UserDeviceDoneRegisterInputLight,
	UserDeviceRegisterInput,
	UserDeviceRegisterOutput,
};

use crate::sentc_app_utils::{check_endpoint_with_app_options, get_app_data_from_req, Endpoint};
use crate::sentc_user_jwt_service::get_jwt_data_from_param;
use crate::sentc_user_service::UserAction;
use crate::user::light::user_light_service;
use crate::user::user_model;

pub(crate) async fn register_light(mut req: Request) -> JRes<RegisterServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let register_input: UserDeviceRegisterInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserRegister)?;

	let out = user_light_service::register_light(&app_data.app_data.app_id, register_input, true).await?;

	echo(out)
}

pub(crate) async fn prepare_register_device_light(mut req: Request) -> JRes<UserDeviceRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: UserDeviceRegisterInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserDeviceRegister)?;

	let out = user_light_service::prepare_register_device_light(&app_data.app_data.app_id, input).await?;

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

pub(crate) async fn done_login_light(mut req: Request) -> JRes<DoneLoginLightOutput>
{
	let body = get_raw_body(&mut req).await?;
	let done_login: DoneLoginServerInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::UserDoneLogin)?;

	let out = user_light_service::done_login_light(app_data, done_login).await?;

	//save the action, only in controller not service because this just not belongs to other controller
	user_model::save_user_action(&app_data.app_data.app_id, &out.user_id, UserAction::Login, 1).await?;

	echo(out)
}

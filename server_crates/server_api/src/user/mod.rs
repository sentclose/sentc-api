pub mod jwt;
pub(crate) mod user_entities;
mod user_model;
pub(crate) mod user_service;

use rustgram::Request;
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::user::{
	ChangePasswordData,
	DoneLoginServerInput,
	DoneLoginServerKeysOutput,
	PrepareLoginSaltServerOutput,
	PrepareLoginServerInput,
	RegisterData,
	RegisterServerOutput,
	ResetPasswordData,
	UserIdentifierAvailableServerInput,
	UserIdentifierAvailableServerOutput,
	UserUpdateServerInput,
	UserUpdateServerOut,
};

use crate::core::api_res::{echo, echo_success, JRes};
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::customer_app::app_util::get_app_data_from_req;
use crate::user::jwt::get_jwt_data_from_param;
use crate::user::user_entities::UserEntity;

pub(crate) async fn exists(mut req: Request) -> JRes<UserIdentifierAvailableServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let data: UserIdentifierAvailableServerInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	let out = user_service::exists(app_data, data).await?;

	echo(out)
}

pub(crate) async fn register(mut req: Request) -> JRes<RegisterServerOutput>
{
	//load the register input from the req body
	let body = get_raw_body(&mut req).await?;
	let register_input: RegisterData = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	let out = user_service::register(app_data, register_input).await?;

	echo(out)
}

pub(crate) async fn prepare_login(mut req: Request) -> JRes<PrepareLoginSaltServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user_identifier: PrepareLoginServerInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	let out = user_service::prepare_login(app_data, user_identifier).await?;

	echo(out)
}

pub(crate) async fn done_login(mut req: Request) -> JRes<DoneLoginServerKeysOutput>
{
	let body = get_raw_body(&mut req).await?;
	let done_login: DoneLoginServerInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	let out = user_service::done_login(app_data, done_login).await?;

	echo(out)
}

//__________________________________________________________________________________________________
// user fn with jwt

pub(crate) async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	let user = get_jwt_data_from_param(&req)?;

	user_service::delete(user).await?;

	echo_success()
}

pub(crate) async fn update(mut req: Request) -> JRes<UserUpdateServerOut>
{
	let body = get_raw_body(&mut req).await?;
	let update_input: UserUpdateServerInput = bytes_to_json(&body)?;
	let user = get_jwt_data_from_param(&req)?;

	let out = user_service::update(user, update_input).await?;

	echo(out)
}

pub(crate) async fn change_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user = get_jwt_data_from_param(&req)?;
	let input: ChangePasswordData = bytes_to_json(&body)?;

	user_service::change_password(user, input).await?;

	echo_success()
}

pub(crate) async fn reset_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user = get_jwt_data_from_param(&req)?; //non fresh jwt here
	let input: ResetPasswordData = bytes_to_json(&body)?;

	user_service::reset_password(user.id.as_str(), input).await?;

	echo_success()
}

pub(crate) async fn get(_req: Request) -> JRes<UserEntity>
{
	let user_id = "abc"; //get this from the url param

	//
	let user = user_model::get_user(user_id).await?;

	echo(user)
}

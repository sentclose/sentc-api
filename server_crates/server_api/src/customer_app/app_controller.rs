use std::future::Future;

use rustgram::Request;
use rustgram_server_util::cache;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, AppRes, JRes, ServerSuccessOutput};
use rustgram_server_util::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};
use server_api_common::app::{
	AppDetails,
	AppFileOptionsInput,
	AppJwtData,
	AppJwtRegisterOutput,
	AppOptions,
	AppRegisterInput,
	AppRegisterOutput,
	AppTokenRenewOutput,
	AppUpdateInput,
};

use crate::customer::{customer_model, customer_util};
use crate::customer_app::app_service::check_file_options;
use crate::customer_app::app_util::{hash_token_to_string, HASH_ALG};
use crate::customer_app::{app_model, app_service, generate_tokens};
use crate::file::file_service;
use crate::sentc_app_entities::AppCustomerAccess;
use crate::sentc_customer_app_service::check_group_options;
use crate::user::jwt::{create_jwt_keys, get_jwt_data_from_param};
use crate::util::api_res::ApiErrorCodes;
use crate::util::{get_app_jwt_sign_key, get_app_jwt_verify_key, APP_TOKEN_CACHE};

pub async fn get_jwt_details(req: Request) -> JRes<Vec<AppJwtData>>
{
	let app_general_data = get_app_general_data(&req)?;

	let jwt_data = app_model::get_jwt_data(&app_general_data.app_id).await?;

	echo(jwt_data)
}

pub async fn get_app_details(req: Request) -> JRes<AppDetails>
{
	let app_general_data = get_app_general_data(&req)?;

	let (details, options, file_options, group_options) = tokio::try_join!(
		app_model::get_app_view(&app_general_data.app_id, app_general_data.owner_type),
		app_model::get_app_options(&app_general_data.app_id),
		app_model::get_app_file_options(&app_general_data.app_id),
		app_model::get_app_group_options(&app_general_data.app_id)
	)?;

	echo(AppDetails {
		options,
		file_options,
		details,
		group_options,
	})
}

pub fn create_app_user(req: Request) -> impl Future<Output = JRes<AppRegisterOutput>>
{
	create_app(req, false)
}

pub fn create_app_group(req: Request) -> impl Future<Output = JRes<AppRegisterOutput>>
{
	create_app(req, true)
}

async fn create_app(mut req: Request, group: bool) -> JRes<AppRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: AppRegisterInput = bytes_to_json(&body)?;

	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	//only create apps when validate the e-mail
	customer_util::check_customer_valid(customer_id).await?;

	let group_id = if group {
		let group_id = get_name_param_from_req(&req, "group_id")?;

		let group_data = customer_model::get_customer_group(group_id, customer_id)
			.await?
			.ok_or_else(|| ServerCoreError::new_msg(400, ApiErrorCodes::GroupAccess, "No access to this group"))?;

		if group_data.rank > 1 {
			return Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::GroupUserRank,
				"No rights to do this action",
			));
		}

		Some(group_id)
	} else {
		None
	};

	let customer_app_data = app_service::create_app(input, customer_id, group_id).await?;

	echo(customer_app_data)
}

pub async fn renew_tokens(req: Request) -> JRes<AppTokenRenewOutput>
{
	//no support for the old tokens anymore (unlike jwt)
	//get the actual tokens (to delete them from the cache)

	let app_general_data = get_app_general_data(&req)?;

	if app_general_data.rank > 2 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"No rights to do this action",
		));
	}

	let (secret_token, public_token) = generate_tokens()?;

	let hashed_secret_token = hash_token_to_string(&secret_token)?;
	let hashed_public_token = hash_token_to_string(&public_token)?;

	app_model::token_renew(
		&app_general_data.app_id,
		hashed_secret_token,
		hashed_public_token,
		HASH_ALG,
	)
	.await?;

	//delete the cache
	let old_hashed_secret = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_secret_token;
	let old_hashed_public_token = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_public_token;

	cache::delete_multiple(&[&old_hashed_secret, &old_hashed_public_token]).await?;

	let out = AppTokenRenewOutput {
		secret_token: base64::encode(secret_token),
		public_token: base64::encode(public_token),
	};

	echo(out)
}

pub async fn add_jwt_keys(req: Request) -> JRes<AppJwtRegisterOutput>
{
	let app_general_data = get_app_general_data(&req)?;

	if app_general_data.rank > 2 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"No rights to do this action",
		));
	}

	let (jwt_sign_key, jwt_verify_key, alg) = create_jwt_keys()?;

	let jwt_id = app_model::add_jwt_keys(
		&app_general_data.app_id,
		jwt_sign_key.as_str(),
		jwt_verify_key.as_str(),
		alg,
	)
	.await?;

	//delete the cache of the app because it can happened that this id was used before
	let verify_key_cache_key = get_app_jwt_verify_key(&jwt_id);
	let sign_key_cache_key = get_app_jwt_sign_key(&jwt_id);
	//delete the app data cache
	let old_hashed_secret = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_secret_token;
	let old_hashed_public_token = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_public_token;

	cache::delete_multiple(&[&verify_key_cache_key, &sign_key_cache_key, &old_hashed_secret, &old_hashed_public_token]).await?;

	echo(AppJwtRegisterOutput {
		app_id: app_general_data.app_id.to_string(),
		jwt_id,
		jwt_verify_key,
		jwt_sign_key,
		jwt_alg: alg.to_string(),
	})
}

pub async fn delete_jwt_keys(req: Request) -> JRes<ServerSuccessOutput>
{
	let app_general_data = get_app_general_data(&req)?;

	if app_general_data.rank > 2 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"No rights to do this action",
		));
	}

	let req_params = get_params(&req)?;
	let jwt_id = get_name_param_from_params(req_params, "jwt_id")?;

	app_model::delete_jwt_keys(&app_general_data.app_id, jwt_id).await?;

	//delete the app data cache

	let old_hashed_secret = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_secret_token;
	let old_hashed_public_token = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_public_token;
	let verify_key_cache_key = get_app_jwt_verify_key(jwt_id);
	let sign_key_cache_key = get_app_jwt_sign_key(jwt_id);

	cache::delete_multiple(&[&old_hashed_secret, &old_hashed_public_token, &verify_key_cache_key, &sign_key_cache_key]).await?;

	echo_success()
}

pub async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	let app_general_data = get_app_general_data(&req)?;

	if app_general_data.rank > 1 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"No rights to do this action",
		));
	}

	app_model::delete(&app_general_data.app_id).await?;

	file_service::delete_file_for_app(&app_general_data.app_id).await?;

	echo_success()
}

pub async fn reset(req: Request) -> JRes<ServerSuccessOutput>
{
	let app_general_data = get_app_general_data(&req)?;

	if app_general_data.rank > 1 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"No rights to do this action",
		));
	}

	app_service::reset(&app_general_data.app_id).await?;

	echo_success()
}

pub async fn update(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	let app_general_data = get_app_general_data(&req)?;

	if app_general_data.rank > 2 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"No rights to do this action",
		));
	}

	let input: AppUpdateInput = bytes_to_json(&body)?;

	app_model::update(&app_general_data.app_id, input.identifier).await?;

	echo_success()
}

pub async fn update_options(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	let app_general_data = get_app_general_data(&req)?;

	if app_general_data.rank > 2 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"No rights to do this action",
		));
	}

	let input: AppOptions = bytes_to_json(&body)?;

	app_model::update_options(&app_general_data.app_id, input).await?;

	//get the public and secret token and delete the app cache because in the cache there are still the old options

	let old_hashed_secret = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_secret_token;
	let old_hashed_public_token = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_public_token;

	cache::delete_multiple(&[&old_hashed_secret, &old_hashed_public_token]).await?;

	echo_success()
}

pub async fn update_file_options(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	let app_general_data = get_app_general_data(&req)?;

	if app_general_data.rank > 2 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"No rights to do this action",
		));
	}

	let input: AppFileOptionsInput = bytes_to_json(&body)?;

	check_file_options(&input)?;

	app_model::update_file_options(&app_general_data.app_id, input).await?;

	//get the public and secret token and delete the app cache because in the cache there are still the old options

	let old_hashed_secret = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_secret_token;
	let old_hashed_public_token = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_public_token;

	cache::delete_multiple(&[&old_hashed_secret, &old_hashed_public_token]).await?;

	echo_success()
}

pub async fn update_group_options(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	let app_general_data = get_app_general_data(&req)?;

	if app_general_data.rank > 2 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"No rights to do this action",
		));
	}

	let input = bytes_to_json(&body)?;

	check_group_options(&input)?;

	app_model::update_group_options(&app_general_data.app_id, input).await?;

	let old_hashed_secret = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_secret_token;
	let old_hashed_public_token = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_public_token;

	cache::delete_multiple(&[&old_hashed_secret, &old_hashed_public_token]).await?;

	echo_success()
}

fn get_app_general_data(req: &Request) -> AppRes<&AppCustomerAccess>
{
	req.extensions()
		.get::<AppCustomerAccess>()
		.ok_or_else(|| ServerCoreError::new_msg(400, ApiErrorCodes::AppTokenNotFound, "No app access"))
}

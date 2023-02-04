use rustgram::Request;
use sentc_crypto_common::server_default::ServerSuccessOutput;
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
use server_api_common::customer::CustomerAppList;
use server_core::cache;
use server_core::input_helper::{bytes_to_json, get_raw_body};
use server_core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};

use crate::customer::customer_util;
use crate::customer_app::app_service::check_file_options;
use crate::customer_app::app_util::{hash_token_to_string, HASH_ALG};
use crate::customer_app::{app_model, app_service, generate_tokens};
use crate::file::file_service;
use crate::user::jwt::{create_jwt_keys, get_jwt_data_from_param};
use crate::util::api_res::{echo, echo_success, ApiErrorCodes, HttpErr, JRes};
use crate::util::{get_app_jwt_sign_key, get_app_jwt_verify_key, APP_TOKEN_CACHE};

pub(crate) async fn get_all_apps(req: Request) -> JRes<Vec<CustomerAppList>>
{
	let user = get_jwt_data_from_param(&req)?;

	let params = get_params(&req)?;
	let last_app_id = get_name_param_from_params(params, "last_app_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let list = app_model::get_all_apps(user.id.to_string(), last_fetched_time, last_app_id.to_string()).await?;

	echo(list)
}

pub(crate) async fn get_jwt_details(req: Request) -> JRes<Vec<AppJwtData>>
{
	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let app_id = get_name_param_from_req(&req, "app_id")?;

	let jwt_data = app_model::get_jwt_data(customer_id.to_string(), app_id.to_string()).await?;

	echo(jwt_data)
}

pub(crate) async fn get_app_details(req: Request) -> JRes<AppDetails>
{
	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let app_id = get_name_param_from_req(&req, "app_id")?;

	let details = app_model::get_app_view(customer_id.clone(), app_id.to_string()).await?;

	let options = app_model::get_app_options(app_id.to_string()).await?;

	let file_options = app_model::get_app_file_options(app_id.to_string()).await?;

	echo(AppDetails {
		options,
		file_options,
		details,
	})
}

pub(crate) async fn create_app(mut req: Request) -> JRes<AppRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: AppRegisterInput = bytes_to_json(&body)?;

	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	//only create apps when validate the e-mail
	customer_util::check_customer_valid(customer_id.to_string()).await?;

	let customer_app_data = app_service::create_app(input, customer_id.to_string()).await?;

	echo(customer_app_data)
}

pub(crate) async fn renew_tokens(req: Request) -> JRes<AppTokenRenewOutput>
{
	//no support for the old tokens anymore (unlike jwt)
	//get the actual tokens (to delete them from the cache)

	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let app_id = get_name_param_from_req(&req, "app_id")?;

	let app_general_data = app_model::get_app_general_data(customer_id.to_string(), app_id.to_string()).await?;

	let (secret_token, public_token) = generate_tokens()?;

	let hashed_secret_token = hash_token_to_string(&secret_token)?;
	let hashed_public_token = hash_token_to_string(&public_token)?;

	app_model::token_renew(
		app_id.to_string(),
		customer_id.to_string(),
		hashed_secret_token,
		hashed_public_token,
		HASH_ALG,
	)
	.await?;

	//delete the cache
	let old_hashed_secret = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_secret_token;
	let old_hashed_public_token = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_public_token;

	cache::delete(old_hashed_secret.as_str()).await;
	cache::delete(old_hashed_public_token.as_str()).await;

	let out = AppTokenRenewOutput {
		secret_token: base64::encode(secret_token),
		public_token: base64::encode(public_token),
	};

	echo(out)
}

pub(crate) async fn add_jwt_keys(req: Request) -> JRes<AppJwtRegisterOutput>
{
	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let app_id = get_name_param_from_req(&req, "app_id")?;

	let (jwt_sign_key, jwt_verify_key, alg) = create_jwt_keys()?;

	let jwt_id = app_model::add_jwt_keys(
		customer_id.to_string(),
		app_id.to_string(),
		jwt_sign_key.as_str(),
		jwt_verify_key.as_str(),
		alg,
	)
	.await?;

	//delete the cache of the app because it can happened that this id was used before
	let verify_key_cache_key = get_app_jwt_verify_key(&jwt_id);
	let sign_key_cache_key = get_app_jwt_sign_key(&jwt_id);
	cache::delete(&verify_key_cache_key).await;
	cache::delete(&sign_key_cache_key).await;

	let out = AppJwtRegisterOutput {
		customer_id: customer_id.to_string(),
		app_id: app_id.to_string(),
		jwt_id,
		jwt_verify_key,
		jwt_sign_key,
		jwt_alg: alg.to_string(),
	};

	//delete the app data cache
	let app_general_data = app_model::get_app_general_data(customer_id.to_string(), app_id.to_string()).await?;

	let old_hashed_secret = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_secret_token;
	let old_hashed_public_token = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_public_token;

	cache::delete(old_hashed_secret.as_str()).await;
	cache::delete(old_hashed_public_token.as_str()).await;

	echo(out)
}

pub(crate) async fn delete_jwt_keys(req: Request) -> JRes<ServerSuccessOutput>
{
	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let req_params = get_params(&req)?;
	let app_id = get_name_param_from_params(req_params, "app_id")?;
	let jwt_id = get_name_param_from_params(req_params, "jwt_id")?;

	app_model::delete_jwt_keys(customer_id.to_string(), app_id.to_string(), jwt_id.to_string()).await?;

	//delete the app data cache
	let app_general_data = app_model::get_app_general_data(customer_id.to_string(), app_id.to_string()).await?;

	let old_hashed_secret = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_secret_token;
	let old_hashed_public_token = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_public_token;

	cache::delete(old_hashed_secret.as_str()).await;
	cache::delete(old_hashed_public_token.as_str()).await;

	let verify_key_cache_key = get_app_jwt_verify_key(jwt_id);
	let sign_key_cache_key = get_app_jwt_sign_key(jwt_id);
	cache::delete(&verify_key_cache_key).await;
	cache::delete(&sign_key_cache_key).await;

	echo_success()
}

pub(crate) async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let app_id = get_name_param_from_req(&req, "app_id")?;

	app_model::delete(customer_id.to_string(), app_id.to_string()).await?;

	file_service::delete_file_for_app(app_id).await?;

	echo_success()
}

pub(crate) async fn update(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: AppUpdateInput = bytes_to_json(&body)?;

	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let app_id = get_name_param_from_req(&req, "app_id")?;

	app_model::update(customer_id.to_string(), app_id.to_string(), input.identifier).await?;

	echo_success()
}

pub(crate) async fn update_options(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: AppOptions = bytes_to_json(&body)?;

	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let app_id = get_name_param_from_req(&req, "app_id")?;

	app_model::update_options(customer_id.to_string(), app_id.to_string(), input).await?;

	//get the public and secret token and delete the app cache because in the cache there are still the old options
	let app_general_data = app_model::get_app_general_data(customer_id.to_string(), app_id.to_string()).await?;

	let old_hashed_secret = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_secret_token;
	let old_hashed_public_token = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_public_token;

	cache::delete(old_hashed_secret.as_str()).await;
	cache::delete(old_hashed_public_token.as_str()).await;

	echo_success()
}

pub(crate) async fn update_file_options(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: AppFileOptionsInput = bytes_to_json(&body)?;

	let app_id = get_name_param_from_req(&req, "app_id")?;
	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	check_file_options(&input)?;

	app_model::update_file_options(customer_id.to_string(), app_id.to_string(), input).await?;

	//get the public and secret token and delete the app cache because in the cache there are still the old options
	let app_general_data = app_model::get_app_general_data(customer_id.to_string(), app_id.to_string()).await?;

	let old_hashed_secret = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_secret_token;
	let old_hashed_public_token = APP_TOKEN_CACHE.to_string() + &app_general_data.hashed_public_token;

	cache::delete(old_hashed_secret.as_str()).await;
	cache::delete(old_hashed_public_token.as_str()).await;

	echo_success()
}

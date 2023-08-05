use std::future::Future;

use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::{AppId, CustomerId, GroupId};
use server_api_common::app::{
	AppFileOptionsInput,
	AppGroupOption,
	AppJwtRegisterOutput,
	AppOptions,
	AppRegisterInput,
	AppRegisterOutput,
	FILE_STORAGE_OWN,
};
use server_api_common::customer::CustomerAppList;

use crate::customer_app::{app_model, generate_tokens};
use crate::file::file_service;
use crate::sentc_app_entities::AppData;
use crate::user::jwt::create_jwt_keys;
use crate::util::api_res::ApiErrorCodes;
use crate::util::{hash_token_to_string, HASH_ALG};
use crate::SENTC_ROOT_APP;

/*
   (
	   secret_token,
	   public_token,
	   hashed_secret_token,
	   hashed_public_token,
	   jwt_sign_key,
	   jwt_verify_key,
	   alg,
   )
*/
type PrepareAppCreateOutput = AppRes<([u8; 50], [u8; 30], String, String, String, String, &'static str)>;

fn prepare_app_create(input: &AppRegisterInput) -> PrepareAppCreateOutput
{
	//1. create and hash tokens
	let (secret_token, public_token) = generate_tokens()?;

	let hashed_secret_token = hash_token_to_string(&secret_token)?;
	let hashed_public_token = hash_token_to_string(&public_token)?;

	//2. create the first jwt keys
	let (jwt_sign_key, jwt_verify_key, alg) = create_jwt_keys()?;

	check_file_options(&input.file_options)?;
	check_group_options(&input.group_options)?;

	Ok((
		secret_token,
		public_token,
		hashed_secret_token,
		hashed_public_token,
		jwt_sign_key,
		jwt_verify_key,
		alg,
	))
}

pub async fn create_sentc_root_app() -> AppRes<()>
{
	let input = AppRegisterInput {
		identifier: None,
		options: AppOptions::default_closed(),
		file_options: AppFileOptionsInput::default_closed(),
		group_options: Default::default(),
	};

	let (_secret_token, _public_token, hashed_secret_token, hashed_public_token, jwt_sign_key, jwt_verify_key, alg) = prepare_app_create(&input)?;

	app_model::create_app_with_id(
		SENTC_ROOT_APP,
		SENTC_ROOT_APP,
		input,
		hashed_secret_token,
		hashed_public_token,
		HASH_ALG,
		&jwt_sign_key,
		&jwt_verify_key,
		alg,
		None::<String>,
	)
	.await?;

	Ok(())
}

pub async fn create_app(
	input: AppRegisterInput,
	customer_id: impl Into<CustomerId>,
	group_id: Option<impl Into<GroupId>>,
) -> AppRes<AppRegisterOutput>
{
	let (secret_token, public_token, hashed_secret_token, hashed_public_token, jwt_sign_key, jwt_verify_key, alg) = prepare_app_create(&input)?;

	//3. create an new app (with new secret_token and public_token)
	//	the str values are used because the real values are exported to the user
	let (app_id, jwt_id) = app_model::create_app(
		customer_id,
		input,
		hashed_secret_token,
		hashed_public_token,
		HASH_ALG,
		&jwt_sign_key,
		&jwt_verify_key,
		alg,
		group_id,
	)
	.await?;

	let customer_app_data = AppRegisterOutput {
		app_id: app_id.to_string(),
		secret_token: base64::encode(secret_token),
		public_token: base64::encode(public_token),
		jwt_data: AppJwtRegisterOutput {
			app_id,
			jwt_id,
			jwt_verify_key,
			jwt_sign_key,
			jwt_alg: alg.to_string(),
		},
	};

	Ok(customer_app_data)
}

pub async fn reset(app_id: impl Into<AppId>) -> AppRes<()>
{
	let app_id = app_id.into();

	app_model::reset(&app_id).await?;

	file_service::delete_file_for_app(app_id).await?;

	Ok(())
}

pub(super) fn check_file_options(input: &AppFileOptionsInput) -> AppRes<()>
{
	//check the file option if the right storage is used
	if input.file_storage > 1 || input.file_storage < -1 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"Wrong storage selected",
		));
	}

	if input.file_storage == FILE_STORAGE_OWN && input.storage_url.is_none() {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"No external storage selected for files",
		));
	}

	if let Some(at) = &input.auth_token {
		if at.len() > 50 {
			return Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::AppAction,
				"Auth token for external storage is too long. Max 50 characters",
			));
		}
	}

	Ok(())
}

pub(super) fn check_group_options(input: &AppGroupOption) -> AppRes<()>
{
	if input.min_rank_key_rotation < 0 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"Wrong rank for the min key rotation rank. Min value is 0.",
		));
	}

	if input.max_key_rotation_month < 0 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::AppAction,
			"Negative values for max monthly key rotations are not allowed.",
		));
	}

	Ok(())
}

pub fn get_all_apps<'a>(
	customer_id: impl Into<CustomerId> + 'a,
	last_fetched_time: u128,
	last_app_id: impl Into<AppId> + 'a,
) -> impl Future<Output = AppRes<Vec<CustomerAppList>>> + 'a
{
	app_model::get_all_apps(customer_id, last_fetched_time, last_app_id)
}

pub fn get_all_apps_group<'a>(
	group_id: impl Into<GroupId> + 'a,
	last_fetched_time: u128,
	last_app_id: impl Into<String> + 'a,
) -> impl Future<Output = AppRes<Vec<CustomerAppList>>> + 'a
{
	app_model::get_all_apps_group(group_id, last_fetched_time, last_app_id)
}

pub fn get_app_data_from_id<'a>(id: impl Into<AppId> + 'a) -> impl Future<Output = AppRes<AppData>> + 'a
{
	app_model::get_app_data_from_id(id)
}

pub fn check_app_exists<'a>(app_id: impl Into<AppId> + 'a, customer_id: impl Into<CustomerId> + 'a) -> impl Future<Output = AppRes<bool>> + 'a
{
	app_model::check_app_exists(app_id, customer_id)
}

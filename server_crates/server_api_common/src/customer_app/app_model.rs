use rustgram_server_util::db::{query, query_first};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::set_params;
use sentc_crypto_common::AppId;
use server_dashboard_common::app::AppOptions;

use crate::customer_app::app_entities::{AppData, AppDataGeneral, AuthWithToken};
use crate::ApiErrorCodes;

pub(super) async fn get_app_options(app_id: impl Into<AppId>) -> AppRes<AppOptions>
{
	//get the options
	//language=SQL
	let sql = r"
SELECT 
    group_create,
    group_get,
    group_user_keys,
    group_user_update_check,
    group_invite,
    group_reject_invite,
    group_accept_invite,
    group_join_req,
    group_accept_join_req,
    group_reject_join_req,
    group_key_rotation,
    group_user_delete,
    group_delete,
    group_leave,
    group_change_rank,
    user_exists,
    user_register,
    user_delete,
    user_update,
    user_change_password,
    user_reset_password,
    user_prepare_login,
    user_done_login,
    user_public_data,
    user_refresh,
    key_register,
    key_get,
    group_auto_invite,
    group_list,
    file_register,
    file_part_upload,
    file_get,
    file_part_download,
    user_device_register,
    user_device_delete,
    user_device_list,
    group_invite_stop,
    user_key_update,
    file_delete,
    content,
    content_small,
    content_med,
    content_large,
    content_x_large,
    user_register_otp,
    user_reset_otp,
    user_disable_otp,
    user_get_otp_recovery_keys
FROM sentc_app_options 
WHERE 
    app_id = ?";

	let options: Option<AppOptions> = query_first(sql, set_params!(app_id.into())).await?;

	let options = match options {
		Some(o) => o,
		None => {
			return Err(ServerCoreError::new_msg(
				401,
				ApiErrorCodes::AppNotFound,
				"App not found",
			))
		},
	};

	Ok(options)
}

async fn get_app_data_private(app_data: AppDataGeneral, auth_with_token: AuthWithToken) -> AppRes<AppData>
{
	//language=SQL
	let sql_jwt = "SELECT id, alg, time FROM sentc_app_jwt_keys WHERE app_id = ? ORDER BY time DESC LIMIT 10";

	//get app file options but without the auth token for external storage
	//language=SQL
	let sql_file_opt = "SELECT file_storage,storage_url FROM sentc_file_options WHERE app_id = ?";

	//get the group options
	//language=SQL
	let sql_group = "SELECT max_key_rotation_month,min_rank_key_rotation FROM sentc_app_group_options WHERE app_id = ?";

	let (jwt_data, options, file_options, group_options) = tokio::try_join!(
		query(sql_jwt, set_params!(app_data.app_id.clone())),
		get_app_options(&app_data.app_id),
		query_first(sql_file_opt, set_params!(app_data.app_id.clone())),
		query_first(sql_group, set_params!(app_data.app_id.clone())),
	)?;

	Ok(AppData {
		app_data,
		jwt_data,
		auth_with_token,
		options,
		file_options: file_options.ok_or_else(|| ServerCoreError::new_msg(401, ApiErrorCodes::AppNotFound, "App not found"))?,
		group_options: group_options.ok_or_else(|| ServerCoreError::new_msg(401, ApiErrorCodes::AppNotFound, "App not found"))?,
	})
}

pub(crate) async fn get_app_data_from_id(id: impl Into<AppId>) -> AppRes<AppData>
{
	//language=SQL
	let sql = r"
SELECT id as app_id, owner_id, owner_type, hashed_secret_token, hashed_public_token, hash_alg, disabled 
FROM sentc_app 
WHERE id = ? LIMIT 1";

	let app_data: AppDataGeneral = query_first(sql, set_params!(id.into()))
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(401, ApiErrorCodes::AppTokenNotFound, "App token not found"))?;

	if app_data.disabled.is_some() {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::AppDisabled,
			"App is disabled and can't be used",
		));
	}

	get_app_data_private(app_data, AuthWithToken::Public).await
}

/**
# Internal app data

cached in the app token middleware
 */
pub(crate) async fn get_app_data(hashed_token: impl Into<String>) -> AppRes<AppData>
{
	let hashed_token = hashed_token.into();

	//language=SQL
	let sql = r"
SELECT id as app_id, owner_id, owner_type, hashed_secret_token, hashed_public_token, hash_alg 
FROM sentc_app 
WHERE hashed_public_token = ? OR hashed_secret_token = ? LIMIT 1";

	let app_data: AppDataGeneral = query_first(sql, set_params!(hashed_token.clone(), hashed_token.clone()))
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(401, ApiErrorCodes::AppTokenNotFound, "App token not found"))?;

	let auth_with_token = if hashed_token == app_data.hashed_public_token {
		AuthWithToken::Public
	} else if hashed_token == app_data.hashed_secret_token {
		AuthWithToken::Secret
	} else {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::AppTokenNotFound,
			"App token not found",
		));
	};

	get_app_data_private(app_data, auth_with_token).await
}

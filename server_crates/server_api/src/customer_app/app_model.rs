use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::db::{exec, exec_transaction, query, query_first, query_string, I32Entity, Params, TransactionData};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::{get_time, set_params};
use sentc_crypto_common::{AppId, CustomerId, GroupId, JwtKeyId, UserId};
use server_api_common::app::{AppFileOptionsInput, AppGroupOption, AppJwtData, AppOptions, AppRegisterInput};
use server_api_common::customer::CustomerAppList;

use crate::customer_app::app_entities::{AppData, AppDataGeneral, AuthWithToken};
use crate::sentc_app_entities::{AppCustomerAccess, CUSTOMER_OWNER_TYPE_GROUP, CUSTOMER_OWNER_TYPE_USER};
use crate::util::api_res::ApiErrorCodes;

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
    content_search,
    file_delete,
    content,
    content_small,
    content_med,
    content_large,
    content_x_large
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
SELECT id as app_id, owner_id, owner_type, hashed_secret_token, hashed_public_token, hash_alg 
FROM sentc_app 
WHERE id = ? LIMIT 1";

	let app_data: AppDataGeneral = query_first(sql, set_params!(id.into()))
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(401, ApiErrorCodes::AppTokenNotFound, "App token not found"))?;

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

/**
Get general app data like internal get app data

but this time with check on app id und customer id

only used internally
 */
pub(crate) async fn get_app_general(app_id: impl Into<AppId>, user_id: impl Into<UserId>) -> AppRes<AppCustomerAccess>
{
	//language=SQL
	let sql = r"
SELECT id as app_id, owner_id, owner_type, hashed_secret_token, hashed_public_token, hash_alg, 'none', 0
FROM sentc_app
WHERE
    id = ? AND
    owner_id = ? AND
    owner_type = 0

UNION

SELECT id as app_id, owner_id, owner_type, hashed_secret_token, hashed_public_token, hash_alg, group_id, `rank` 
FROM sentc_app, sentc_group_user 
WHERE 
    id = ? AND
    owner_type = 1 AND
    user_id = ? AND 
    group_id = owner_id
";

	let user_id = user_id.into();
	let app_id = app_id.into();

	query_first(sql, set_params!(app_id.clone(), user_id.clone(), app_id, user_id))
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(401, ApiErrorCodes::AppTokenNotFound, "App token not found"))
}

/**
Get jwt data like internal get app data

but this time check with customer and app id and not limited
*/
pub(super) async fn get_jwt_data(app_id: impl Into<AppId>) -> AppRes<Vec<AppJwtData>>
{
	//language=SQL
	let sql = r"
SELECT ak.id, alg, ak.time, sign_key, verify_key 
FROM sentc_app a, sentc_app_jwt_keys ak 
WHERE 
    app_id = ? AND 
    app_id = a.id
ORDER BY ak.time DESC";

	let mut jwt_data: Vec<AppJwtData> = query(sql, set_params!(app_id.into())).await?;

	let key = encrypted_at_rest_root::get_key_map().await;

	for jwt_datum in &mut jwt_data {
		jwt_datum.sign_key = encrypted_at_rest_root::decrypt_with_key(&key, &jwt_datum.sign_key)?;
	}

	Ok(jwt_data)
}

pub(super) async fn get_all_apps(
	customer_id: impl Into<CustomerId>,
	last_fetched_time: u128,
	last_app_id: impl Into<AppId>,
) -> AppRes<Vec<CustomerAppList>>
{
	//language=SQL
	let sql = r"
SELECT * FROM (
SELECT id, identifier, time, null as group_name
FROM sentc_app 
WHERE 
    owner_id = ? AND 
    owner_type = 0

UNION 

SELECT id, identifier, sentc_app.time, name as group_name 
FROM sentc_app, sentc_group_user, sentc_customer_group 
WHERE
      owner_type = 1 AND
      user_id = ? AND
      group_id = owner_id AND 
      sentc_group_id = group_id 
) as apps"
		.to_string();

	let customer_id = customer_id.into();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " WHERE time >=? AND (time > ? OR (time = ? AND id > ?)) ORDER BY time, id LIMIT 20";
		(
			sql,
			set_params!(
				customer_id.clone(),
				customer_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_app_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY time, id LIMIT 20";
		(sql, set_params!(customer_id.clone(), customer_id,))
	};

	let list: Vec<CustomerAppList> = query_string(sql, params).await?;

	Ok(list)
}

pub(super) async fn get_all_apps_group(
	group_id: impl Into<GroupId>,
	last_fetched_time: u128,
	last_app_id: impl Into<AppId>,
) -> AppRes<Vec<CustomerAppList>>
{
	//language=SQL
	let sql = r"
SELECT id, identifier, sentc_app.time, name as group_name 
FROM sentc_app, sentc_customer_group 
WHERE
      owner_type = 1 AND
      owner_id = ? AND
      sentc_group_id = owner_id"
		.to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND sentc_app.time >=? AND (sentc_app.time > ? OR (sentc_app.time = ? AND id > ?)) ORDER BY sentc_app.time, id LIMIT 20";
		(
			sql,
			set_params!(
				group_id.into(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_app_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY sentc_app.time, id LIMIT 20";
		(sql, set_params!(group_id.into()))
	};

	let list: Vec<CustomerAppList> = query_string(sql, params).await?;

	Ok(list)
}

pub(super) async fn get_app_view(app_id: impl Into<AppId>, owner_type: i32) -> AppRes<CustomerAppList>
{
	//tanks to the app access middleware we know the owner type
	let sql = if owner_type == CUSTOMER_OWNER_TYPE_GROUP {
		//language=SQL
		r"
SELECT id, identifier, sentc_app.time, name as group_name 
FROM sentc_app, sentc_customer_group 
WHERE
      owner_type = 1 AND
      id = ? AND
      sentc_group_id = owner_id 
		"
	} else {
		//language=SQL
		r"SELECT id, identifier, time, null as group_name FROM sentc_app WHERE id = ?"
	};

	query_first(sql, set_params!(app_id.into()))
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(400, ApiErrorCodes::AppNotFound, "App not found"))
}

pub(super) async fn get_app_file_options(app_id: impl Into<AppId>) -> AppRes<AppFileOptionsInput>
{
	//language=SQL
	let sql = "SELECT file_storage,storage_url,auth_token FROM sentc_file_options WHERE app_id = ?";

	query_first(sql, set_params!(app_id.into()))
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(400, ApiErrorCodes::AppNotFound, "App not found"))
}

pub(super) async fn get_app_group_options(app_id: impl Into<AppId>) -> AppRes<AppGroupOption>
{
	//language=SQL
	let sql = "SELECT max_key_rotation_month,min_rank_key_rotation FROM sentc_app_group_options WHERE app_id = ?";

	query_first(sql, set_params!(app_id.into()))
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(400, ApiErrorCodes::AppNotFound, "App not found"))
}

pub(super) async fn check_app_exists(app_id: impl Into<AppId>, customer_id: impl Into<CustomerId>) -> AppRes<bool>
{
	//language=SQL
	let sql = "SELECT 1 FROM sentc_app WHERE owner_id = ? AND id = ?";

	let exists: Option<I32Entity> = query_first(sql, set_params!(customer_id.into(), app_id.into())).await?;

	Ok(exists.is_some())
}

//__________________________________________________________________________________________________

pub(super) async fn create_app_with_id(
	app_id: impl Into<AppId>,
	customer_id: impl Into<CustomerId>,
	input: AppRegisterInput,
	hashed_secret_token: String,
	hashed_public_token: String,
	alg: impl Into<String>,
	first_jwt_sign_key: impl Into<String>,
	first_jwt_verify_key: impl Into<String>,
	first_jwt_alg: impl Into<String>,
	group_id: Option<impl Into<GroupId>>,
) -> AppRes<JwtKeyId>
{
	//in another fn to also create the sentc root app with a specific id
	//for normal apps use the create app fn

	let app_id = app_id.into();
	let time = get_time()?;

	//language=SQL
	let sql_app = r"
INSERT INTO sentc_app 
    (id, 
     owner_id, 
     owner_type,
     identifier, 
     hashed_secret_token, 
     hashed_public_token, 
     hash_alg,
     time
     ) 
VALUES (?,?,?,?,?,?,?,?)";

	let identifier = input.identifier.unwrap_or_default();

	let (owner_id, owner_type) = match group_id {
		Some(id) => (id.into(), CUSTOMER_OWNER_TYPE_GROUP),
		None => (customer_id.into(), CUSTOMER_OWNER_TYPE_USER),
	};

	let params_app = set_params!(
		app_id.clone(),
		owner_id,
		owner_type,
		identifier,
		hashed_secret_token,
		hashed_public_token,
		alg.into(),
		time.to_string()
	);

	let jwt_key_id = create_id();

	let encrypted_sign_key = encrypted_at_rest_root::encrypt(&first_jwt_sign_key.into()).await?;

	//language=SQL
	let sql_jwt = "INSERT INTO sentc_app_jwt_keys (id, app_id, sign_key, verify_key, alg, time) VALUES (?,?,?,?,?,?)";
	let params_jwt = set_params!(
		jwt_key_id.clone(),
		app_id.clone(),
		encrypted_sign_key,
		first_jwt_verify_key.into(),
		first_jwt_alg.into(),
		time.to_string()
	);

	let (sql_options, params_options) = prepare_options_insert(app_id.clone(), input.options);

	//language=SQL
	let sql_file_options = "INSERT INTO sentc_file_options (app_id, file_storage, storage_url, auth_token) VALUES (?,?,?,?)";
	let params_file_options = set_params!(
		app_id.clone(),
		input.file_options.file_storage,
		input.file_options.storage_url,
		input.file_options.auth_token
	);

	//language=SQL
	let sql_group_options = "INSERT INTO sentc_app_group_options (app_id, max_key_rotation_month, min_rank_key_rotation) VALUES (?,?,?)";
	let params_group_options = set_params!(
		app_id,
		input.group_options.max_key_rotation_month,
		input.group_options.min_rank_key_rotation
	);

	exec_transaction(vec![
		TransactionData {
			sql: sql_app,
			params: params_app,
		},
		TransactionData {
			sql: sql_jwt,
			params: params_jwt,
		},
		TransactionData {
			sql: sql_options,
			params: params_options,
		},
		TransactionData {
			sql: sql_file_options,
			params: params_file_options,
		},
		TransactionData {
			sql: sql_group_options,
			params: params_group_options,
		},
	])
	.await?;

	Ok(jwt_key_id)
}

pub(super) async fn create_app(
	customer_id: impl Into<CustomerId>,
	input: AppRegisterInput,
	hashed_secret_token: String,
	hashed_public_token: String,
	alg: impl Into<String>,
	first_jwt_sign_key: impl Into<String>,
	first_jwt_verify_key: impl Into<String>,
	first_jwt_alg: impl Into<String>,
	group_id: Option<impl Into<GroupId>>,
) -> AppRes<(AppId, JwtKeyId)>
{
	let app_id = create_id();

	let jwt_key_id = create_app_with_id(
		app_id.clone(),
		customer_id,
		input,
		hashed_secret_token,
		hashed_public_token,
		alg,
		first_jwt_sign_key,
		first_jwt_verify_key,
		first_jwt_alg,
		group_id,
	)
	.await?;

	Ok((app_id, jwt_key_id))
}

pub(super) async fn token_renew(
	app_id: impl Into<AppId>,
	hashed_secret_token: String,
	hashed_public_token: String,
	alg: impl Into<String>,
) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_app SET hashed_secret_token = ?, hashed_public_token = ?, hash_alg = ? WHERE id = ?";

	exec(
		sql,
		set_params!(hashed_secret_token, hashed_public_token, alg.into(), app_id.into()),
	)
	.await?;

	Ok(())
}

pub(super) async fn add_jwt_keys(
	app_id: impl Into<AppId>,
	new_jwt_sign_key: impl Into<String>,
	new_jwt_verify_key: impl Into<String>,
	new_jwt_alg: impl Into<String>,
) -> AppRes<JwtKeyId>
{
	let time = get_time()?;
	let jwt_key_id = create_id();

	let encrypted_sign_key = encrypted_at_rest_root::encrypt(&new_jwt_sign_key.into()).await?;

	//language=SQL
	let sql = "INSERT INTO sentc_app_jwt_keys (id, app_id, sign_key, verify_key, alg, time) VALUES (?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			jwt_key_id.clone(),
			app_id.into(),
			encrypted_sign_key,
			new_jwt_verify_key.into(),
			new_jwt_alg.into(),
			time.to_string()
		),
	)
	.await?;

	Ok(jwt_key_id)
}

pub(super) async fn delete_jwt_keys(app_id: impl Into<AppId>, jwt_key_id: impl Into<JwtKeyId>) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_app_jwt_keys WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(jwt_key_id.into(), app_id.into())).await?;

	Ok(())
}

pub(super) async fn update(app_id: impl Into<AppId>, identifier: Option<String>) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_app SET identifier = ? WHERE id = ?";

	let identifier = identifier.unwrap_or_default();

	exec(sql, set_params!(identifier, app_id.into())).await?;

	Ok(())
}

pub(super) async fn update_options(app_id: impl Into<AppId>, app_options: AppOptions) -> AppRes<()>
{
	let app_id = app_id.into();

	//delete the old options

	//language=SQL
	let sql = "DELETE FROM sentc_app_options WHERE app_id = ?";

	exec(sql, set_params!(app_id.clone())).await?;

	let (sql_options, params_options) = prepare_options_insert(app_id, app_options);

	exec(sql_options, params_options).await?;

	Ok(())
}

pub(super) async fn update_file_options(app_id: impl Into<AppId>, options: AppFileOptionsInput) -> AppRes<()>
{
	let app_id = app_id.into();

	//language=SQL
	let sql = "UPDATE sentc_file_options SET storage_url = ?, file_storage = ?, auth_token = ? WHERE app_id = ?";

	exec(
		sql,
		set_params!(options.storage_url, options.file_storage, options.auth_token, app_id),
	)
	.await?;

	Ok(())
}

pub(super) async fn update_group_options(app_id: impl Into<AppId>, options: AppGroupOption) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_app_group_options SET max_key_rotation_month = ?, min_rank_key_rotation = ? WHERE app_id = ?";

	exec(
		sql,
		set_params!(
			options.max_key_rotation_month,
			options.min_rank_key_rotation,
			app_id.into()
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn delete(app_id: impl Into<AppId>) -> AppRes<()>
{
	//delete the rest with trigger

	let app_id = app_id.into();

	//language=SQL
	let sql = "DELETE FROM sentc_app WHERE id = ?";

	exec(sql, set_params!(app_id)).await?;

	Ok(())
}

pub(super) async fn reset(app_id: impl Into<AppId>) -> AppRes<()>
{
	let app_id = app_id.into();

	/*
	1. delete all users
	2. delete all groups
	3. delete all keys
	4. delete content

	Do not delete the options.
	 */

	//language=SQL
	let sql_user = r"DELETE FROM sentc_user WHERE app_id = ?";
	let params_user = set_params!(app_id.clone());

	//language=SQL
	let sql_group = r"DELETE FROM sentc_group WHERE app_id = ?";
	let params_group = set_params!(app_id.clone());

	//language=SQL
	let sql_keys = r"DELETE FROM sentc_sym_key_management WHERE app_id = ?";
	let params_keys = set_params!(app_id.clone());

	//language=SQL
	let sql_content = r"DELETE FROM sentc_content WHERE app_id = ?";
	let params_content = set_params!(app_id.clone());

	exec_transaction(vec![
		TransactionData {
			sql: sql_user,
			params: params_user,
		},
		TransactionData {
			sql: sql_group,
			params: params_group,
		},
		TransactionData {
			sql: sql_keys,
			params: params_keys,
		},
		TransactionData {
			sql: sql_content,
			params: params_content,
		},
	])
	.await?;

	Ok(())
}

//__________________________________________________________________________________________________

fn prepare_options_insert(app_id: impl Into<AppId>, app_options: AppOptions) -> (&'static str, Params)
{
	//language=SQL
	let sql = r"
INSERT INTO sentc_app_options 
    (
     app_id, 
     group_create, 
     group_get, 
     group_user_keys,
     group_user_update_check,
     group_invite, 
     group_auto_invite,
     group_reject_invite, 
     group_accept_invite, 
     group_join_req, 
     group_accept_join_req, 
     group_reject_join_req, 
     group_key_rotation, 
     group_user_delete, 
     group_change_rank, 
     group_delete, 
     group_leave, 
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
     content_search,
     file_delete,
     content,
     content_small,
     content_med,
     content_large,
     content_x_large
     ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

	let params_options = set_params!(
		app_id.into(),
		app_options.group_create,
		app_options.group_get,
		app_options.group_user_keys,
		app_options.group_user_update_check,
		app_options.group_invite,
		app_options.group_auto_invite,
		app_options.group_reject_invite,
		app_options.group_accept_invite,
		app_options.group_join_req,
		app_options.group_accept_join_req,
		app_options.group_reject_join_req,
		app_options.group_key_rotation,
		app_options.group_user_delete,
		app_options.group_change_rank,
		app_options.group_delete,
		app_options.group_leave,
		app_options.user_exists,
		app_options.user_register,
		app_options.user_delete,
		app_options.user_update,
		app_options.user_change_password,
		app_options.user_reset_password,
		app_options.user_prepare_login,
		app_options.user_done_login,
		app_options.user_public_data,
		app_options.user_jwt_refresh,
		app_options.key_register,
		app_options.key_get,
		app_options.group_list,
		app_options.file_register,
		app_options.file_part_upload,
		app_options.file_get,
		app_options.file_part_download,
		app_options.user_device_register,
		app_options.user_device_delete,
		app_options.user_device_list,
		app_options.group_invite_stop,
		app_options.user_key_update,
		app_options.content_search,
		app_options.file_delete,
		app_options.content,
		app_options.content_small,
		app_options.content_med,
		app_options.content_large,
		app_options.content_x_large,
	);

	(sql, params_options)
}

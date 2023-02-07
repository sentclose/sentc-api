use sentc_crypto_common::{AppId, JwtKeyId};
use server_api_common::app::{AppFileOptionsInput, AppJwtData, AppOptions, AppRegisterInput};
use server_api_common::customer::CustomerAppList;
use server_core::db::{exec, exec_transaction, query, query_first, query_string, I64Entity, Params, TransactionData};
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::res::AppRes;
use server_core::{get_time, set_params, str_clone, str_get, str_t, u128_get};
use uuid::Uuid;

use crate::customer_app::app_entities::{AppData, AppDataGeneral, AppJwt, AuthWithToken};
use crate::sentc_app_entities::AppFileOptions;
use crate::util::api_res::ApiErrorCodes;

pub(super) async fn get_app_options(app_id: str_t!()) -> AppRes<AppOptions>
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
    content_search
FROM sentc_app_options 
WHERE 
    app_id = ?";

	let options: Option<AppOptions> = query_first(sql, set_params!(str_get!(app_id))).await?;

	let options = match options {
		Some(o) => o,
		None => {
			return Err(SentcCoreError::new_msg(
				401,
				ApiErrorCodes::AppNotFound,
				"App not found",
			))
		},
	};

	Ok(options)
}

/**
# Internal app data

cached in the app token middleware
*/
pub(crate) async fn get_app_data(hashed_token: str_t!()) -> AppRes<AppData>
{
	let hashed_token = str_get!(hashed_token);

	//language=SQL
	let sql = r"
SELECT id as app_id, customer_id, hashed_secret_token, hashed_public_token, hash_alg 
FROM sentc_app 
WHERE hashed_public_token = ? OR hashed_secret_token = ? LIMIT 1";

	let app_data: Option<AppDataGeneral> = query_first(sql, set_params!(str_clone!(hashed_token), str_clone!(hashed_token))).await?;

	let app_data = match app_data {
		Some(d) => d,
		None => {
			return Err(SentcCoreError::new_msg(
				401,
				ApiErrorCodes::AppTokenNotFound,
				"App token not found",
			))
		},
	};

	//language=SQL
	let sql = "SELECT id, alg, time FROM sentc_app_jwt_keys WHERE app_id = ? ORDER BY time DESC LIMIT 10";

	let jwt_data: Vec<AppJwt> = query(sql, set_params!(str_clone!(&app_data.app_id))).await?;

	let auth_with_token = if hashed_token == app_data.hashed_public_token {
		AuthWithToken::Public
	} else if hashed_token == app_data.hashed_secret_token {
		AuthWithToken::Secret
	} else {
		return Err(SentcCoreError::new_msg(
			401,
			ApiErrorCodes::AppTokenNotFound,
			"App token not found",
		));
	};

	let options = get_app_options(&app_data.app_id).await?;

	//get app file options but without the auth token for external storage
	//language=SQL
	let sql = "SELECT file_storage,storage_url FROM sentc_file_options WHERE app_id = ?";
	let file_options: Option<AppFileOptions> = query_first(sql, set_params!(str_clone!(&app_data.app_id))).await?;

	let file_options = match file_options {
		Some(o) => o,
		None => {
			return Err(SentcCoreError::new_msg(
				401,
				ApiErrorCodes::AppNotFound,
				"App not found",
			))
		},
	};

	Ok(AppData {
		app_data,
		jwt_data,
		auth_with_token,
		options,
		file_options,
	})
}

/**
Get general app data like internal get app data

but this time with check on app id und customer id

only used internally
*/
pub(super) async fn get_app_general_data(customer_id: str_t!(), app_id: str_t!()) -> AppRes<AppDataGeneral>
{
	//language=SQL
	let sql = r"
SELECT id as app_id, customer_id, hashed_secret_token, hashed_public_token, hash_alg 
FROM sentc_app 
WHERE customer_id = ? AND id = ? LIMIT 1";

	let app_data: Option<AppDataGeneral> = query_first(sql, set_params!(str_get!(customer_id), str_get!(app_id))).await?;

	match app_data {
		Some(d) => Ok(d),
		None => {
			Err(SentcCoreError::new_msg(
				401,
				ApiErrorCodes::AppTokenNotFound,
				"App token not found",
			))
		},
	}
}

/**
Get jwt data like internal get app data

but this time check with customer and app id and not limited
*/
pub(super) async fn get_jwt_data(customer_id: str_t!(), app_id: str_t!()) -> AppRes<Vec<AppJwtData>>
{
	//language=SQL
	let sql = r"
SELECT ak.id, alg, ak.time, sign_key, verify_key 
FROM sentc_app a, sentc_app_jwt_keys ak 
WHERE 
    app_id = ? AND 
    customer_id = ? AND 
    app_id = a.id 
ORDER BY ak.time DESC";

	let jwt_data: Vec<AppJwtData> = query(sql, set_params!(str_get!(app_id), str_get!(customer_id))).await?;

	Ok(jwt_data)
}

pub(super) async fn get_all_apps(customer_id: str_t!(), last_fetched_time: u128, last_app_id: str_t!()) -> AppRes<Vec<CustomerAppList>>
{
	//language=SQL
	let sql = "SELECT id,identifier, time FROM sentc_app WHERE customer_id = ?".to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND time >=? AND (time > ? OR (time = ? AND id > ?)) ORDER BY time, id LIMIT 20";
		(
			sql,
			set_params!(
				str_get!(customer_id),
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				str_get!(last_app_id)
			),
		)
	} else {
		let sql = sql + " ORDER BY time, id LIMIT 20";
		(sql, set_params!(str_get!(customer_id)))
	};

	let list: Vec<CustomerAppList> = query_string(sql, params).await?;

	Ok(list)
}

pub(super) async fn get_app_view(customer_id: str_t!(), app_id: str_t!()) -> AppRes<CustomerAppList>
{
	//language=SQL
	let sql = "SELECT id,identifier, time FROM sentc_app WHERE customer_id = ? AND id = ?";

	let out: Option<CustomerAppList> = query_first(sql, set_params!(str_get!(customer_id), str_get!(app_id))).await?;

	match out {
		Some(o) => Ok(o),
		None => {
			Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::AppNotFound,
				"App not found",
			))
		},
	}
}

pub(super) async fn get_app_file_options(app_id: str_t!()) -> AppRes<AppFileOptionsInput>
{
	//language=SQL
	let sql = "SELECT file_storage,storage_url,auth_token FROM sentc_file_options WHERE app_id = ?";

	let out: Option<AppFileOptionsInput> = query_first(sql, set_params!(str_get!(app_id))).await?;

	match out {
		Some(o) => Ok(o),
		None => {
			Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::AppNotFound,
				"App not found",
			))
		},
	}
}

//__________________________________________________________________________________________________

pub(super) async fn create_app(
	customer_id: str_t!(),
	input: AppRegisterInput,
	hashed_secret_token: String,
	hashed_public_token: String,
	alg: str_t!(),
	first_jwt_sign_key: str_t!(),
	first_jwt_verify_key: str_t!(),
	first_jwt_alg: str_t!(),
) -> AppRes<(AppId, JwtKeyId)>
{
	let app_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//language=SQL
	let sql_app = r"
INSERT INTO sentc_app 
    (id, 
     customer_id, 
     identifier, 
     hashed_secret_token, 
     hashed_public_token, 
     hash_alg,
     time
     ) 
VALUES (?,?,?,?,?,?,?)";

	let identifier = match input.identifier {
		Some(i) => i,
		None => "".to_string(),
	};

	let params_app = set_params!(
		str_clone!(&app_id),
		str_get!(customer_id),
		identifier,
		hashed_secret_token,
		hashed_public_token,
		str_get!(alg),
		u128_get!(time)
	);

	let jwt_key_id = Uuid::new_v4().to_string();

	//language=SQL
	let sql_jwt = "INSERT INTO sentc_app_jwt_keys (id, app_id, sign_key, verify_key, alg, time) VALUES (?,?,?,?,?,?)";
	let params_jwt = set_params!(
		str_clone!(&jwt_key_id),
		str_clone!(&app_id),
		str_get!(first_jwt_sign_key),
		str_get!(first_jwt_verify_key),
		str_get!(first_jwt_alg),
		u128_get!(time)
	);

	let (sql_options, params_options) = prepare_options_insert(str_clone!(&app_id), input.options);

	//language=SQL
	let sql_file_options = "INSERT INTO sentc_file_options (app_id, file_storage, storage_url, auth_token) VALUES (?,?,?,?)";
	let params_file_options = set_params!(
		str_clone!(&app_id),
		input.file_options.file_storage,
		input.file_options.storage_url,
		input.file_options.auth_token
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
	])
	.await?;

	Ok((app_id, jwt_key_id))
}

pub(super) async fn token_renew(
	app_id: str_t!(),
	customer_id: str_t!(),
	hashed_secret_token: String,
	hashed_public_token: String,
	alg: str_t!(),
) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_app SET hashed_secret_token = ?, hashed_public_token = ?, hash_alg = ? WHERE id = ? AND customer_id = ?";

	exec(
		sql,
		set_params!(
			hashed_secret_token,
			hashed_public_token,
			str_get!(alg),
			str_get!(app_id),
			str_get!(customer_id)
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn add_jwt_keys(
	customer_id: str_t!(),
	app_id: str_t!(),
	new_jwt_sign_key: str_t!(),
	new_jwt_verify_key: str_t!(),
	new_jwt_alg: str_t!(),
) -> AppRes<JwtKeyId>
{
	let app_id = str_get!(app_id);

	check_app_exists(customer_id, str_clone!(app_id)).await?;

	let time = get_time()?;
	let jwt_key_id = Uuid::new_v4().to_string();

	//language=SQL
	let sql = "INSERT INTO sentc_app_jwt_keys (id, app_id, sign_key, verify_key, alg, time) VALUES (?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			str_clone!(&jwt_key_id),
			app_id,
			str_get!(new_jwt_sign_key),
			str_get!(new_jwt_verify_key),
			str_get!(new_jwt_alg),
			u128_get!(time)
		),
	)
	.await?;

	Ok(jwt_key_id)
}

pub(super) async fn delete_jwt_keys(customer_id: str_t!(), app_id: str_t!(), jwt_key_id: str_t!()) -> AppRes<()>
{
	let app_id = str_get!(app_id);

	check_app_exists(customer_id, str_clone!(app_id)).await?;

	//language=SQL
	let sql = "DELETE FROM sentc_app_jwt_keys WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(str_get!(jwt_key_id), app_id)).await?;

	Ok(())
}

pub(super) async fn update(customer_id: str_t!(), app_id: str_t!(), identifier: Option<String>) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_app SET identifier = ? WHERE customer_id = ? AND id = ?";

	let identifier = match identifier {
		Some(i) => i,
		None => "".to_string(),
	};

	exec(sql, set_params!(identifier, str_get!(customer_id), str_get!(app_id))).await?;

	Ok(())
}

pub(super) async fn update_options(customer_id: str_t!(), app_id: str_t!(), app_options: AppOptions) -> AppRes<()>
{
	let app_id = str_get!(app_id);

	check_app_exists(customer_id, str_clone!(app_id)).await?;

	//delete the old options

	//language=SQL
	let sql = "DELETE FROM sentc_app_options WHERE app_id = ?";

	exec(sql, set_params!(str_clone!(app_id))).await?;

	let (sql_options, params_options) = prepare_options_insert(app_id, app_options);

	exec(sql_options, params_options).await?;

	Ok(())
}

pub(super) async fn update_file_options(customer_id: str_t!(), app_id: str_t!(), options: AppFileOptionsInput) -> AppRes<()>
{
	let app_id = str_get!(app_id);

	check_app_exists(customer_id, str_clone!(app_id)).await?;

	//language=SQL
	let sql = "UPDATE sentc_file_options SET storage_url = ?, file_storage = ?, auth_token = ? WHERE app_id = ?";

	exec(
		sql,
		set_params!(options.storage_url, options.file_storage, options.auth_token, app_id),
	)
	.await?;

	Ok(())
}

pub(super) async fn delete(customer_id: str_t!(), app_id: str_t!()) -> AppRes<()>
{
	//use the double check with the customer id to check if this app really belongs to the customer!

	//language=SQL
	let sql = "DELETE FROM sentc_app WHERE customer_id = ? AND id = ?";

	exec(sql, set_params!(str_get!(customer_id), str_get!(app_id))).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn check_app_exists(customer_id: str_t!(), app_id: str_t!()) -> AppRes<()>
{
	//check if this app belongs to this customer
	//language=SQL
	let sql = "SELECT 1 FROM sentc_app WHERE id = ? AND customer_id = ?";
	let app_exists: Option<I64Entity> = query_first(sql, set_params!(str_get!(app_id), str_get!(customer_id))).await?;

	match app_exists {
		Some(_) => {},
		None => {
			return Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::AppNotFound,
				"App not found in this user space",
			))
		},
	}

	Ok(())
}

fn prepare_options_insert(app_id: str_t!(), app_options: AppOptions) -> (&'static str, Params)
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
     content_search
     ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

	let params_options = set_params!(
		str_get!(app_id),
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
		app_options.content_search
	);

	(sql, params_options)
}

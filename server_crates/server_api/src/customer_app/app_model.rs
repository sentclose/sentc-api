use sentc_crypto_common::{AppId, CustomerId, JwtKeyId, UserId};
use server_api_common::app::{AppFileOptions, AppJwtData, AppOptions, AppRegisterInput};
use server_core::db::{exec, exec_transaction, query, query_first, Params, TransactionData};
use server_core::{get_time, set_params};
use uuid::Uuid;

use crate::customer_app::app_entities::{AppData, AppDataGeneral, AppExistsEntity, AppJwt, AuthWithToken};
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

/**
# Internal app data

cached in the app token middleware
*/
pub(crate) async fn get_app_data(hashed_token: &str) -> AppRes<AppData>
{
	//language=SQL
	let sql = r"
SELECT id as app_id, customer_id, hashed_secret_token, hashed_public_token, hash_alg 
FROM sentc_app 
WHERE hashed_public_token = ? OR hashed_secret_token = ? LIMIT 1";

	let app_data: Option<AppDataGeneral> = query_first(sql, set_params!(hashed_token.to_string(), hashed_token.to_string())).await?;

	let app_data = match app_data {
		Some(d) => d,
		None => {
			return Err(HttpErr::new(
				401,
				ApiErrorCodes::AppTokenNotFound,
				"App token not found".to_string(),
				None,
			))
		},
	};

	//language=SQL
	let sql = "SELECT id, alg, time FROM sentc_app_jwt_keys WHERE app_id = ? ORDER BY time DESC LIMIT 10";

	let jwt_data: Vec<AppJwt> = query(sql, set_params!(app_data.app_id.to_string())).await?;

	let auth_with_token = if hashed_token == app_data.hashed_public_token {
		AuthWithToken::Public
	} else if hashed_token == app_data.hashed_secret_token {
		AuthWithToken::Secret
	} else {
		return Err(HttpErr::new(
			401,
			ApiErrorCodes::AppTokenNotFound,
			"App token not found".to_string(),
			None,
		));
	};

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
    group_invite_stop
FROM sentc_app_options 
WHERE 
    app_id = ?";

	let options: Option<AppOptions> = query_first(sql, set_params!(app_data.app_id.to_string())).await?;

	let options = match options {
		Some(o) => o,
		None => {
			return Err(HttpErr::new(
				401,
				ApiErrorCodes::AppTokenNotFound,
				"App token not found".to_string(),
				None,
			))
		},
	};

	//get app file options
	//language=SQL
	let sql = "SELECT file_storage,storage_url FROM sentc_file_options WHERE app_id = ?";
	let file_options: Option<AppFileOptions> = query_first(sql, set_params!(app_data.app_id.to_string())).await?;

	let file_options = match file_options {
		Some(o) => o,
		None => {
			return Err(HttpErr::new(
				401,
				ApiErrorCodes::AppTokenNotFound,
				"App token not found".to_string(),
				None,
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
pub(super) async fn get_app_general_data(customer_id: CustomerId, app_id: AppId) -> AppRes<AppDataGeneral>
{
	//language=SQL
	let sql = r"
SELECT id as app_id, customer_id, hashed_secret_token, hashed_public_token, hash_alg 
FROM sentc_app 
WHERE customer_id = ? AND id = ? LIMIT 1";

	let app_data: Option<AppDataGeneral> = query_first(sql, set_params!(customer_id, app_id)).await?;

	match app_data {
		Some(d) => Ok(d),
		None => {
			return Err(HttpErr::new(
				401,
				ApiErrorCodes::AppTokenNotFound,
				"App token not found".to_string(),
				None,
			))
		},
	}
}

/**
Get jwt data like internal get app data

but this time check with customer and app id and not limited
*/
pub(super) async fn get_jwt_data(customer_id: CustomerId, app_id: AppId) -> AppRes<Vec<AppJwtData>>
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

	let jwt_data: Vec<AppJwtData> = query(sql, set_params!(app_id, customer_id)).await?;

	Ok(jwt_data)
}

pub(super) async fn create_app(
	customer_id: &UserId,
	input: AppRegisterInput,
	hashed_secret_token: String,
	hashed_public_token: String,
	alg: &str,
	first_jwt_sign_key: &str,
	first_jwt_verify_key: &str,
	first_jwt_alg: &str,
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
		app_id.to_string(),
		customer_id.to_string(),
		identifier,
		hashed_secret_token.to_string(),
		hashed_public_token.to_string(),
		alg.to_string(),
		time.to_string()
	);

	let jwt_key_id = Uuid::new_v4().to_string();

	//language=SQL
	let sql_jwt = "INSERT INTO sentc_app_jwt_keys (id, app_id, sign_key, verify_key, alg, time) VALUES (?,?,?,?,?,?)";
	let params_jwt = set_params!(
		jwt_key_id.to_string(),
		app_id.to_string(),
		first_jwt_sign_key.to_string(),
		first_jwt_verify_key.to_string(),
		first_jwt_alg.to_string(),
		time.to_string()
	);

	let (sql_options, params_options) = prepare_options_insert(app_id.to_string(), input.options);

	//language=SQL
	let sql_file_options = "INSERT INTO sentc_file_options (app_id, file_storage, storage_url) VALUES (?,?,?)";
	let params_file_options = set_params!(
		app_id.to_string(),
		input.file_options.file_storage,
		input.file_options.storage_url
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
	app_id: AppId,
	customer_id: CustomerId,
	hashed_secret_token: String,
	hashed_public_token: String,
	alg: &str,
) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_app SET hashed_secret_token = ?, hashed_public_token = ?, hash_alg = ? WHERE id = ? AND customer_id = ?";

	exec(
		sql,
		set_params!(
			hashed_secret_token,
			hashed_public_token,
			alg.to_string(),
			app_id,
			customer_id
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn add_jwt_keys(
	customer_id: CustomerId,
	app_id: AppId,
	new_jwt_sign_key: &str,
	new_jwt_verify_key: &str,
	new_jwt_alg: &str,
) -> AppRes<JwtKeyId>
{
	check_app_exists(customer_id, app_id.to_string()).await?;

	let time = get_time()?;
	let jwt_key_id = Uuid::new_v4().to_string();

	//language=SQL
	let sql = "INSERT INTO sentc_app_jwt_keys (id, app_id, sign_key, verify_key, alg, time) VALUES (?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			jwt_key_id.to_string(),
			app_id.to_string(),
			new_jwt_sign_key.to_string(),
			new_jwt_verify_key.to_string(),
			new_jwt_alg.to_string(),
			time.to_string()
		),
	)
	.await?;

	Ok(jwt_key_id)
}

pub(super) async fn delete_jwt_keys(customer_id: CustomerId, app_id: AppId, jwt_key_id: JwtKeyId) -> AppRes<()>
{
	check_app_exists(customer_id, app_id.to_string()).await?;

	//language=SQL
	let sql = "DELETE FROM sentc_app_jwt_keys WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(jwt_key_id, app_id)).await?;

	Ok(())
}

pub(super) async fn update(customer_id: CustomerId, app_id: AppId, identifier: Option<String>) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_app SET identifier = ? WHERE customer_id = ? AND id = ?";

	let identifier = match identifier {
		Some(i) => i,
		None => "".to_string(),
	};

	exec(sql, set_params!(identifier, customer_id, app_id)).await?;

	Ok(())
}

pub(super) async fn update_options(customer_id: CustomerId, app_id: AppId, app_options: AppOptions) -> AppRes<()>
{
	check_app_exists(customer_id, app_id.to_string()).await?;

	//delete the old options

	//language=SQL
	let sql = "DELETE FROM sentc_app_options WHERE app_id = ?";

	exec(sql, set_params!(app_id.to_string())).await?;

	let (sql_options, params_options) = prepare_options_insert(app_id, app_options);

	exec(sql_options, params_options).await?;

	Ok(())
}

pub(super) async fn update_file_options(customer_id: CustomerId, app_id: AppId, options: AppFileOptions) -> AppRes<()>
{
	check_app_exists(customer_id, app_id.to_string()).await?;

	//language=SQL
	let sql = "UPDATE sentc_file_options SET storage_url = ?, file_storage = ? WHERE app_id = ?";

	exec(sql, set_params!(options.storage_url, options.file_storage, app_id)).await?;

	Ok(())
}

pub(super) async fn delete(customer_id: CustomerId, app_id: AppId) -> AppRes<()>
{
	//use the double check with the customer id to check if this app really belongs to the customer!

	//language=SQL
	let sql = "DELETE FROM sentc_app WHERE customer_id = ? AND id = ?";

	exec(sql, set_params!(customer_id, app_id)).await?;

	Ok(())
}

//__________________________________________________________________________________________________

async fn check_app_exists(customer_id: CustomerId, app_id: AppId) -> AppRes<()>
{
	//check if this app belongs to this customer
	//language=SQL
	let sql = "SELECT 1 FROM sentc_app WHERE id = ? AND customer_id = ?";
	let app_exists: Option<AppExistsEntity> = query_first(sql, set_params!(app_id, customer_id)).await?;

	match app_exists {
		Some(_) => {},
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::AppNotFound,
				"App not found in this user space".to_string(),
				None,
			))
		},
	}

	Ok(())
}

fn prepare_options_insert(app_id: AppId, app_options: AppOptions) -> (&'static str, Params)
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
     group_invite_stop
     ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

	let params_options = set_params!(
		app_id.to_string(),
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
		app_options.group_invite_stop
	);

	(sql, params_options)
}

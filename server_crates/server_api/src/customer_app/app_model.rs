use sentc_crypto_common::{AppId, CustomerId, UserId};
use uuid::Uuid;

use crate::core::api_res::{ApiErrorCodes, HttpErr};
use crate::core::db::{exec, exec_transaction, query, query_first, TransactionData};
use crate::core::get_time;
use crate::customer_app::app_entities::{AppData, AppDataGeneral, AppJwt, AuthWithToken};
use crate::set_params;

pub(crate) async fn get_app_data(hashed_token: &str) -> Result<AppData, HttpErr>
{
	//language=SQL
	let sql = r"
SELECT id as app_id, customer_id, hashed_secret_token, hashed_public_token, hash_alg 
FROM app 
WHERE hashed_public_token = ? OR hashed_secret_token = ? LIMIT 1";

	let app_data: Option<AppDataGeneral> = query_first(
		sql.to_string(),
		set_params!(hashed_token.to_string(), hashed_token.to_string()),
	)
	.await?;

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
	let sql = "SELECT id, alg, time FROM app_jwt_keys WHERE app_id = ? ORDER BY time DESC LIMIT 10";

	let jwt_data: Vec<AppJwt> = query(sql.to_string(), set_params!(app_data.app_id.to_string())).await?;

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

	Ok(AppData {
		app_data,
		jwt_data,
		auth_with_token,
	})
}

pub(super) async fn get_app_general_data(customer_id: CustomerId, app_id: AppId) -> Result<AppDataGeneral, HttpErr>
{
	//language=SQL
	let sql = r"
SELECT id as app_id, customer_id, hashed_secret_token, hashed_public_token, hash_alg 
FROM app 
WHERE customer_id = ? AND id = ? LIMIT 1";

	let app_data: Option<AppDataGeneral> = query_first(sql.to_string(), set_params!(customer_id, app_id)).await?;

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

pub(super) async fn create_app(
	customer_id: &UserId,
	identifier: Option<String>,
	hashed_secret_token: String,
	hashed_public_token: String,
	alg: &str,
	first_jwt_sign_key: &str,
	first_jwt_verify_key: &str,
	first_jwt_alg: &str,
) -> Result<String, HttpErr>
{
	let app_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//language=SQL
	let sql_app = r"
INSERT INTO app 
    (id, 
     customer_id, 
     identifier, 
     hashed_secret_token, 
     hashed_public_token, 
     hash_alg,
     time
     ) 
VALUES (?,?,?,?,?,?,?)";

	let identifier = match identifier {
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
	let sql_jwt = "INSERT INTO app_jwt_keys (id, app_id, sign_key, verify_key, alg, time) VALUES (?,?,?,?,?,?)";
	let params_jwt = set_params!(
		jwt_key_id,
		app_id.to_string(),
		first_jwt_sign_key.to_string(),
		first_jwt_verify_key.to_string(),
		first_jwt_alg.to_string(),
		time.to_string()
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
	])
	.await?;

	Ok(app_id)
}

pub(super) async fn token_renew(
	app_id: AppId,
	customer_id: CustomerId,
	hashed_secret_token: String,
	hashed_public_token: String,
	alg: &str,
) -> Result<(), HttpErr>
{
	//language=SQL
	let sql = "UPDATE app SET hashed_secret_token = ?, hashed_public_token = ?, hash_alg = ? WHERE id = ? AND customer_id = ?";

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

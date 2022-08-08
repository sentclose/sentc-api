use sentc_crypto_common::crypto::GeneratedSymKeyHeadServerInput;
use sentc_crypto_common::{AppId, SymKeyId};
use uuid::Uuid;

use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::core::db::{exec, query_first, query_string};
use crate::core::get_time;
use crate::key_management::key_entity::SymKeyEntity;
use crate::set_params;

pub(super) async fn register_sym_key(app_id: AppId, input: GeneratedSymKeyHeadServerInput) -> AppRes<SymKeyId>
{
	let key_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//language=SQL
	let sql = r"
INSERT INTO sentc_sym_key_management 
    (
     id, 
     app_id, 
     master_key_id, 
     encrypted_key, 
     master_key_alg, 
     time
     ) 
VALUES (?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			key_id.to_string(),
			app_id,
			input.master_key_id,
			input.encrypted_key_string,
			input.alg,
			time.to_string()
		),
	)
	.await?;

	Ok(key_id)
}

pub(super) async fn get_sym_key_by_id(app_id: AppId, key_id: SymKeyId) -> AppRes<SymKeyEntity>
{
	//language=SQL
	let sql = "SELECT * FROM sentc_sym_key_management WHERE app_id = ? AND id = ?";

	let key: Option<SymKeyEntity> = query_first(sql, set_params!(app_id, key_id)).await?;

	match key {
		Some(k) => Ok(k),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::KeyNotFound,
				"Key not found".to_string(),
				None,
			))
		},
	}
}

pub(super) async fn get_all_sym_keys_to_master_key(
	app_id: AppId,
	master_key_id: SymKeyId,
	last_fetched_time: u128,
	last_id: SymKeyId,
) -> AppRes<Vec<SymKeyEntity>>
{
	//language=SQL
	let sql = "SELECT * FROM sentc_sym_key_management WHERE app_id = ? AND master_key_id = ?".to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND time <= ? AND (time < ? OR (time = ? AND id > ?)) ORDER BY time DESC, id LIMIT 50";

		(
			sql,
			set_params!(
				app_id,
				master_key_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id
			),
		)
	} else {
		let sql = sql + " ORDER BY time DESC, id LIMIT 50";

		(sql, set_params!(app_id, master_key_id))
	};

	let keys: Vec<SymKeyEntity> = query_string(sql, params).await?;

	Ok(keys)
}

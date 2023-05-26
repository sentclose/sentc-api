use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::db::{exec, query_first, query_string};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::{get_time, set_params};
use sentc_crypto_common::crypto::GeneratedSymKeyHeadServerInput;
use sentc_crypto_common::{AppId, SymKeyId, UserId};

use crate::key_management::key_entity::SymKeyEntity;
use crate::util::api_res::ApiErrorCodes;

pub(super) async fn register_sym_key(
	app_id: impl Into<AppId>,
	creator_id: impl Into<UserId>,
	input: GeneratedSymKeyHeadServerInput,
) -> AppRes<SymKeyId>
{
	let key_id = create_id();
	let time = get_time()?;

	//language=SQL
	let sql = r"
INSERT INTO sentc_sym_key_management 
    (
     id, 
     app_id, 
     master_key_id, 
     creator_id,
     encrypted_key, 
     master_key_alg, 
     time
     ) 
VALUES (?,?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			key_id.clone(),
			app_id.into(),
			input.master_key_id,
			creator_id.into(),
			input.encrypted_key_string,
			input.alg,
			time.to_string()
		),
	)
	.await?;

	Ok(key_id)
}

pub(super) async fn delete_sym_key(app_id: impl Into<AppId>, creator_id: impl Into<UserId>, key_id: impl Into<SymKeyId>) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_sym_key_management WHERE app_id = ? AND creator_id = ? AND id = ?";

	exec(sql, set_params!(app_id.into(), creator_id.into(), key_id.into())).await?;

	Ok(())
}

pub(super) async fn get_sym_key_by_id(app_id: impl Into<AppId>, key_id: impl Into<SymKeyId>) -> AppRes<SymKeyEntity>
{
	//language=SQL
	let sql = "SELECT id, master_key_id, encrypted_key, master_key_alg, time FROM sentc_sym_key_management WHERE app_id = ? AND id = ?";

	let key: Option<SymKeyEntity> = query_first(sql, set_params!(app_id.into(), key_id.into())).await?;

	match key {
		Some(k) => Ok(k),
		None => {
			Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::KeyNotFound,
				"Key not found",
			))
		},
	}
}

pub(super) async fn get_all_sym_keys_to_master_key(
	app_id: impl Into<AppId>,
	master_key_id: impl Into<SymKeyId>,
	last_fetched_time: u128,
	last_id: impl Into<SymKeyId>,
) -> AppRes<Vec<SymKeyEntity>>
{
	//language=SQL
	let sql = "SELECT id, master_key_id, encrypted_key, master_key_alg, time FROM sentc_sym_key_management WHERE app_id = ? AND master_key_id = ?"
		.to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND time <= ? AND (time < ? OR (time = ? AND id > ?)) ORDER BY time DESC, id LIMIT 50";

		(
			sql,
			set_params!(
				app_id.into(),
				master_key_id.into(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY time DESC, id LIMIT 50";

		(sql, set_params!(app_id.into(), master_key_id.into(),))
	};

	let keys: Vec<SymKeyEntity> = query_string(sql, params).await?;

	Ok(keys)
}

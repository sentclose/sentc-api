use sentc_crypto_common::{AppId, FileId, SymKeyId, UserId};
use server_core::db::{exec, exec_transaction, query, query_first, query_string, TransactionData};
use server_core::file::FilePartListToDelete;
use server_core::{get_time, set_params};
use uuid::Uuid;

use crate::file::file_entities::{FileMetaData, FilePartListItem, FileSessionCheck};
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

static MAX_CHUNK_SIZE: usize = 5 * 1024 * 1024;
static MAX_SESSION_ALIVE_TIME: u128 = 24 * 60 * 60 * 1000;

pub(super) async fn register_file(
	key_id: SymKeyId,
	belongs_to_id: Option<String>,
	belongs_to_type: i32,
	app_id: AppId,
	user_id: UserId,
) -> AppRes<(String, String)>
{
	let file_id = Uuid::new_v4().to_string();
	let session_id = Uuid::new_v4().to_string();

	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_file (id, owner, belongs_to, belongs_to_type, app_id, key_id, time) VALUES (?,?,?,?,?,?,?)";
	let params = set_params!(
		file_id.to_string(),
		user_id,
		belongs_to_id,
		belongs_to_type,
		app_id.to_string(),
		key_id,
		time.to_string()
	);

	//language=SQL
	let sql_session = "INSERT INTO sentc_file_session (id, file_id, app_id, created_at, expected_size, max_chunk_size) VALUES (?,?,?,?,?,?)";
	let params_session = set_params!(
		session_id.to_string(),
		file_id.to_string(),
		app_id,
		time.to_string(),
		0,
		MAX_CHUNK_SIZE.to_string()
	);

	exec_transaction(vec![
		TransactionData {
			sql,
			params,
		},
		TransactionData {
			sql: sql_session,
			params: params_session,
		},
	])
	.await?;

	Ok((file_id, session_id))
}

pub(super) async fn check_session(app_id: AppId, session_id: String, user_id: UserId) -> AppRes<(FileId, usize)>
{
	//language=SQL
	let sql = r"
SELECT file_id, created_at, max_chunk_size 
FROM 
    sentc_file f, 
    sentc_file_session s 
WHERE 
    f.id = file_id AND 
    s.id = ? AND 
    owner = ? AND 
    f.app_id = ?";

	let check: Option<FileSessionCheck> = query_first(sql, set_params!(session_id.to_string(), user_id, app_id.to_string())).await?;

	let check = match check {
		Some(o) => o,
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::FileSessionNotFound,
				"File upload session not found".to_string(),
				None,
			));
		},
	};

	//check the exp date
	let time = get_time()?;

	if check.created_at + MAX_SESSION_ALIVE_TIME < time {
		//session exp
		delete_session(session_id, app_id).await?;

		return Err(HttpErr::new(
			400,
			ApiErrorCodes::FileSessionExpired,
			"File upload session expired".to_string(),
			None,
		));
	}

	Ok((check.file_id, check.max_chunk_size))
}

pub(super) async fn save_part(app_id: AppId, file_id: FileId, size: usize, sequence: i32, end: bool, extern_storage: bool) -> AppRes<()>
{
	let part_id = Uuid::new_v4().to_string();

	//language=SQL
	let sql = "INSERT INTO sentc_file_part (id, file_id, app_id, size, sequence, extern) VALUES (?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			part_id,
			file_id.to_string(),
			app_id.to_string(),
			size.to_string(),
			sequence,
			extern_storage
		),
	)
	.await?;

	if end {
		//language=SQL
		let sql = "DELETE FROM sentc_file_session WHERE app_id = ? AND file_id = ?";
		exec(sql, set_params!(app_id, file_id)).await?;
	}

	Ok(())
}

pub(super) async fn delete_session(session_id: String, app_id: AppId) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_file_session WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(session_id, app_id)).await?;

	Ok(())
}

pub(super) async fn get_file(app_id: AppId, file_id: FileId) -> AppRes<FileMetaData>
{
	//language=SQL
	let sql = "SELECT id, owner, belongs_to, belongs_to_type, key_id, time FROM sentc_file WHERE app_id = ? AND id = ?";

	let file: Option<FileMetaData> = query_first(sql, set_params!(app_id, file_id)).await?;

	match file {
		Some(f) => Ok(f),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::FileNotFound,
				"File not found".to_string(),
				None,
			))
		},
	}
}

pub(super) async fn get_file_parts(app_id: AppId, file_id: FileId, last_sequence: i32) -> AppRes<Vec<FilePartListItem>>
{
	//get the file parts
	//language=SQL
	let sql = r"
SELECT id,sequence,extern 
FROM 
    sentc_file_part 
WHERE 
    app_id = ? AND 
    file_id = ?"
		.to_string();

	let (sql, params) = if last_sequence > 0 {
		let sql = sql + " AND sequence > ? ORDER BY sequence LIMIT 500";
		(sql, set_params!(app_id, file_id, last_sequence))
	} else {
		let sql = sql + " ORDER BY sequence LIMIT 500";
		(sql, set_params!(app_id, file_id))
	};

	let file_parts: Vec<FilePartListItem> = query_string(sql, params).await?;

	if file_parts.len() == 0 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::FileNotFound,
			"File not found".to_string(),
			None,
		));
	}

	Ok(file_parts)
}

//__________________________________________________________________________________________________

pub(super) async fn get_file_parts_to_delete(app_id: AppId, file_id: FileId) -> AppRes<(Vec<FilePartListToDelete>, Vec<FilePartListItem>)>
{
	//language=SQL
	let sql = r"
SELECT 
    id,
    extern 
FROM sentc_file_part 
WHERE 
    app_id = ? AND 
    file_id = ? AND 
    extern = 0 
ORDER BY sequence";

	let file_parts: Vec<FilePartListToDelete> = query(sql, set_params!(app_id.to_string(), file_id.to_string())).await?;

	//language=SQL
	let sql = r"
SELECT 
    id,
    extern 
FROM sentc_file_part 
WHERE 
    app_id = ? AND 
    file_id = ? AND 
    extern = 1 
ORDER BY sequence";

	let extern_file_parts: Vec<FilePartListItem> = query(sql, set_params!(app_id, file_id)).await?;

	Ok((file_parts, extern_file_parts))
}

pub(super) async fn delete_file(app_id: AppId, file_id: FileId) -> AppRes<()>
{
	//make sure to delete the parts first

	//language=SQL
	let sql = "DELETE FROM sentc_file WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(file_id, app_id)).await?;

	Ok(())
}

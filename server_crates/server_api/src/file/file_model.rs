use sentc_crypto_common::{AppId, CustomerId, FileId, FileSessionId, GroupId, PartId, SymKeyId, UserId};
use server_core::db::{exec, exec_string, exec_transaction, get_in, query_first, query_string, TransactionData, TupleEntity};
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::res::AppRes;
use server_core::{get_time, set_params, set_params_vec, set_params_vec_outer};
use uuid::Uuid;

use crate::file::file_entities::{FileExternalStorageUrl, FileMetaData, FilePartListItem, FilePartListItemDelete, FileSessionCheck};
use crate::file::file_service::FILE_BELONGS_TO_TYPE_GROUP;
use crate::util::api_res::ApiErrorCodes;

const MAX_CHUNK_SIZE: usize = 5 * 1024 * 1024;
const MAX_SESSION_ALIVE_TIME: u128 = 24 * 60 * 60 * 1000;
const FILE_STATUS_AVAILABLE: i32 = 1;
const FILE_STATUS_TO_DELETE: i32 = 0;
//const FILE_STATUS_DISABLED: i32 = 1;

pub(super) async fn register_file(
	key_id: SymKeyId,
	master_key_id: String,
	file_name: Option<String>,
	belongs_to_id: Option<String>,
	belongs_to_type: i32,
	app_id: impl Into<AppId>,
	user_id: impl Into<UserId>,
) -> AppRes<(String, String)>
{
	let app_id = app_id.into();

	let file_id = Uuid::new_v4().to_string();
	let session_id = Uuid::new_v4().to_string();

	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_file (id, owner, belongs_to, belongs_to_type, app_id, key_id, time, status, delete_at, encrypted_file_name, master_key_id) VALUES (?,?,?,?,?,?,?,?,?,?,?)";
	let params = set_params!(
		file_id.clone(),
		user_id.into(),
		belongs_to_id,
		belongs_to_type,
		app_id.clone(),
		key_id,
		time.to_string(),
		FILE_STATUS_AVAILABLE,
		0,
		file_name,
		master_key_id
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

pub(super) async fn check_session(
	app_id: impl Into<AppId>,
	session_id: impl Into<FileSessionId>,
	user_id: impl Into<UserId>,
) -> AppRes<(FileId, usize)>
{
	let app_id = app_id.into();
	let session_id = session_id.into();

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

	let check: Option<FileSessionCheck> = query_first(sql, set_params!(session_id.clone(), user_id.into(), app_id.clone())).await?;

	let check = match check {
		Some(o) => o,
		None => {
			return Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::FileSessionNotFound,
				"File upload session not found",
			));
		},
	};

	//check the exp date
	let time = get_time()?;

	if check.created_at + MAX_SESSION_ALIVE_TIME < time {
		//session exp
		delete_session(session_id, app_id).await?;

		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::FileSessionExpired,
			"File upload session expired",
		));
	}

	Ok((check.file_id, check.max_chunk_size))
}

pub(super) async fn save_part(
	app_id: impl Into<AppId>,
	file_id: FileId,
	part_id: String,
	size: usize,
	sequence: i32,
	end: bool,
	extern_storage: bool,
) -> AppRes<()>
{
	//part and file id are owned because they are fetched

	let app_id = app_id.into();

	//language=SQL
	let sql = "INSERT INTO sentc_file_part (id, file_id, app_id, size, sequence, extern) VALUES (?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			part_id,
			file_id.clone(),
			app_id.clone(),
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

pub(super) async fn delete_file_part(app_id: impl Into<AppId>, part_id: impl Into<PartId>) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_file_part WHERE app_id = ? AND id = ?";

	exec(sql, set_params!(app_id.into(), part_id.into())).await?;

	Ok(())
}

pub(super) async fn delete_session(session_id: impl Into<FileSessionId>, app_id: impl Into<AppId>) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_file_session WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(session_id.into(), app_id.into())).await?;

	Ok(())
}

pub(super) async fn get_file(app_id: impl Into<AppId>, file_id: impl Into<FileId>) -> AppRes<FileMetaData>
{
	//language=SQL
	let sql = r"
SELECT 
    id, 
    owner, 
    belongs_to, 
    belongs_to_type, 
    key_id, 
    time, 
    encrypted_file_name,
    master_key_id
FROM sentc_file 
WHERE 
    app_id = ? AND 
    id = ? AND 
    status = ?";

	let file: Option<FileMetaData> = query_first(sql, set_params!(app_id.into(), file_id.into(), FILE_STATUS_AVAILABLE)).await?;

	match file {
		Some(f) => Ok(f),
		None => {
			Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::FileNotFound,
				"File not found",
			))
		},
	}
}

pub(super) async fn get_file_parts(app_id: impl Into<AppId>, file_id: impl Into<FileId>, last_sequence: i32) -> AppRes<Vec<FilePartListItem>>
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
		(sql, set_params!(app_id.into(), file_id.into(), last_sequence))
	} else {
		let sql = sql + " ORDER BY sequence LIMIT 500";
		(sql, set_params!(app_id.into(), file_id.into()))
	};

	let file_parts: Vec<FilePartListItem> = query_string(sql, params).await?;

	if file_parts.is_empty() {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::FileNotFound,
			"File not found",
		));
	}

	Ok(file_parts)
}

pub(super) async fn update_file_name(file_name: Option<String>, app_id: impl Into<AppId>, file_id: impl Into<FileId>) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_file SET encrypted_file_name = ? WHERE app_id = ? AND id = ?";

	exec(sql, set_params!(file_name, app_id.into(), file_id.into())).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn delete_file(app_id: impl Into<AppId>, file_id: impl Into<FileId>) -> AppRes<()>
{
	//mark the file as to delete
	let time = get_time()?;

	//language=SQL
	let sql = "UPDATE sentc_file SET status = ?, delete_at = ? WHERE id = ? AND app_id = ?";

	exec(
		sql,
		set_params!(FILE_STATUS_TO_DELETE, time.to_string(), file_id.into(), app_id.into()),
	)
	.await?;

	Ok(())
}

pub(super) async fn delete_files_for_customer_group(group_id: impl Into<GroupId>) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = r"
UPDATE 
    sentc_file 
SET 
    status = ?, 
    delete_at = ? 
WHERE 
    app_id IN (
    SELECT 
        sentc_app.id 
    FROM sentc_app 
    WHERE owner_id = ? AND 
          owner_type = 1
    )";

	exec(
		sql,
		set_params!(FILE_STATUS_TO_DELETE, time.to_string(), group_id.into()),
	)
	.await?;

	Ok(())
}

pub(super) async fn delete_files_for_customer(customer_id: impl Into<CustomerId>) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = r"
UPDATE 
    sentc_file 
SET 
    status = ?, 
    delete_at = ? 
WHERE 
    app_id IN (
    SELECT 
        sentc_app.id 
    FROM sentc_app 
    WHERE owner_id = ? AND 
          owner_type = 0
    )";

	exec(
		sql,
		set_params!(FILE_STATUS_TO_DELETE, time.to_string(), customer_id.into()),
	)
	.await?;

	Ok(())
}

pub(super) async fn delete_files_for_app(app_id: impl Into<AppId>) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = "UPDATE sentc_file SET status = ?, delete_at = ? WHERE app_id = ?";

	exec(
		sql,
		set_params!(FILE_STATUS_TO_DELETE, time.to_string(), app_id.into()),
	)
	.await?;

	Ok(())
}

pub(super) async fn delete_files_for_group(app_id: impl Into<AppId>, group_id: impl Into<GroupId>, children: Vec<String>) -> AppRes<()>
{
	let app_id = app_id.into();
	let time = get_time()?;

	//language=SQL
	let sql = r"
UPDATE 
    sentc_file 
SET
    status = ?, 
    delete_at = ? 
WHERE 
    app_id = ? AND 
    belongs_to_type = ? AND  
	belongs_to = ?";

	exec(
		sql,
		set_params!(
			FILE_STATUS_TO_DELETE,
			time.to_string(),
			app_id.clone(),
			FILE_BELONGS_TO_TYPE_GROUP,
			group_id.into(),
		),
	)
	.await?;

	//update children, can't use mysql recursion here, because it says the rec table doesn't exist
	if !children.is_empty() {
		let get_in = get_in(&children);

		//language=SQLx
		let sql = format!(
			"UPDATE sentc_file SET status = ?, delete_at = ? WHERE app_id = ? AND belongs_to_type = ? AND belongs_to IN ({})",
			get_in
		);

		let mut exec_vec = Vec::with_capacity(children.len() + 4);

		exec_vec.push(TupleEntity(FILE_STATUS_TO_DELETE.to_string()));
		exec_vec.push(TupleEntity(time.to_string()));
		exec_vec.push(TupleEntity(app_id));
		exec_vec.push(TupleEntity(FILE_BELONGS_TO_TYPE_GROUP.to_string()));

		for child in children {
			exec_vec.push(TupleEntity(child));
		}

		exec_string(sql, set_params_vec!(exec_vec)).await?;
	}

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn get_external_app_file_delete_info(app_ids: Vec<AppId>) -> AppRes<Vec<FileExternalStorageUrl>>
{
	let ins = get_in(&app_ids);

	//language=SQLx
	let sql = format!(
		"SELECT storage_url,app_id,auth_token FROM sentc_file_options WHERE app_id IN ({})",
		ins
	);

	let res: Vec<FileExternalStorageUrl> = query_string(sql, set_params_vec_outer!(app_ids)).await?;

	Ok(res)
}

pub(super) async fn get_all_files_marked_to_delete(last_part_id: Option<String>, start_time: u128) -> AppRes<Vec<FilePartListItemDelete>>
{
	//owned last part id because of the file worker

	//language=SQL
	let sql = r"
SELECT fp.id as file_id_part_id, sequence, extern, fp.app_id as part_app_id 
FROM 
    sentc_file_part fp, sentc_file f 
WHERE 
    status = ? AND 
    file_id = f.id AND 
    delete_at < ?"
		.to_string();

	let (sql, params) = match last_part_id {
		None => {
			let sql = sql + " ORDER BY fp.id LIMIT 500";
			(sql, set_params!(FILE_STATUS_TO_DELETE, start_time.to_string()))
		},
		Some(last) => {
			let sql = sql + " AND fp.id > ? ORDER BY fp.id LIMIT 500";
			(sql, set_params!(FILE_STATUS_TO_DELETE, start_time.to_string(), last))
		},
	};

	let file_parts: Vec<FilePartListItemDelete> = query_string(sql, params).await?;

	Ok(file_parts)
}

pub(super) async fn delete_file_complete(start_time: u128) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_file WHERE delete_at < ? AND status = ?";

	exec(sql, set_params!(start_time.to_string(), FILE_STATUS_TO_DELETE)).await?;

	Ok(())
}

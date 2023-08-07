use std::collections::HashMap;
use std::time::Duration;

use rustgram_server_util::get_time;
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::{AppId, PartId};

use crate::file_entities::{FileExternalStorageUrl, FilePartListItemDelete};
use crate::file_model;

pub async fn start() -> AppRes<()>
{
	//get all files which are marked as to delete
	let start_time = get_time()?;

	let mut last_id = None;

	loop {
		let parts = file_model::get_all_files_marked_to_delete(last_id, start_time).await?;
		let part_len = parts.len();

		match parts.last() {
			Some(p) => last_id = Some(p.part_id.to_string()),
			None => {
				//parts are empty
				break;
			},
		}

		//wait here for the response! not like send email, because this worker is started in a tokio task.
		delete_parts(parts).await?;

		if part_len < 500 {
			break;
		}
	}

	//now delete all files which got a smaller deleted_at time as the start time
	file_model::delete_file_complete(start_time).await?;

	Ok(())
}

async fn delete_parts(parts: Vec<FilePartListItemDelete>) -> AppRes<()>
{
	//split extern and intern
	let mut intern_storage = Vec::with_capacity(parts.len());

	let mut extern_storage_map: HashMap<AppId, Vec<PartId>> = HashMap::new();

	for part in parts {
		if part.extern_storage {
			//split the app ids
			extern_storage_map
				.entry(part.app_id.to_string())
				.or_default()
				.push(part.part_id);
		} else {
			intern_storage.push(part.part_id);
		}
	}

	if !intern_storage.is_empty() {
		rustgram_server_util::file::delete_parts(&intern_storage).await?;
	}

	if !extern_storage_map.is_empty() {
		delete_external(extern_storage_map).await?;
	}

	Ok(())
}

async fn delete_external(map: HashMap<AppId, Vec<PartId>>) -> AppRes<()>
{
	//get the app info
	let app_ids = map.keys().cloned().collect::<Vec<AppId>>();

	let app_info = file_model::get_external_app_file_delete_info(app_ids).await?;

	//iterate over each app which has files to delete from their external storage
	for (app_id, file_data) in map {
		let (url, auth_token) = match find_app_info(app_id, &app_info) {
			Some(u) => (u.0, u.1),
			None => continue,
		};

		//make req to the external storage url delete endpoint
		// with the part ids in body as json
		let body = match serde_json::to_string(&file_data) {
			Err(_e) => continue,
			Ok(f) => f,
		};

		let client = reqwest::Client::new();

		// header token
		let req = client.post(url).body(body).timeout(Duration::from_secs(10));

		let req = match auth_token {
			Some(at) => req.header("x-sentc-app-token", at),
			None => req,
		};

		//don't wait for the res
		tokio::task::spawn(req.send());
	}

	Ok(())
}

fn find_app_info(app_id: AppId, app_info: &Vec<FileExternalStorageUrl>) -> Option<(&str, &Option<String>)>
{
	for info in app_info {
		if info.app_id == app_id {
			return Some((&info.storage_url, &info.auth_token));
		}
	}

	None
}

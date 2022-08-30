use crate::file::file_entities::FilePartListItem;
use crate::file::file_model;
use crate::util::api_res::AppRes;

pub async fn start() -> AppRes<()>
{
	//get all files which are marked as to delete

	let mut last_id = None;

	loop {
		let parts = file_model::get_all_files_marked_to_delete(last_id).await?;
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

	Ok(())
}

async fn delete_parts(parts: Vec<FilePartListItem>) -> AppRes<()>
{
	//split extern and intern
	let mut extern_storage = Vec::with_capacity(parts.len());
	let mut intern_storage = Vec::with_capacity(parts.len());

	for part in parts {
		if part.extern_storage {
			extern_storage.push(part.part_id);
		} else {
			intern_storage.push(part.part_id);
		}
	}

	server_core::file::delete_parts(&intern_storage).await?;

	//TODO make a req to delete the extern parts

	Ok(())
}

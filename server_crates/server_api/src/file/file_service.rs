use sentc_crypto_common::file::{BelongsToType, FileRegisterInput, FileRegisterOutput};
use sentc_crypto_common::{AppId, UserId};

use crate::file::file_model;
use crate::util::api_res::AppRes;

pub async fn register_file(input: FileRegisterInput, app_id: AppId, user_id: UserId) -> AppRes<FileRegisterOutput>
{
	//check first if belongs to is set

	let belongs_to_type = match input.belongs_to_type {
		BelongsToType::None => 0,
		BelongsToType::Group => {
			//check if the user got access to this group
			1
		},
		BelongsToType::User => {
			//check if the other user is in this app
			2
		},
	};

	let (file_id, session_id) = file_model::register_file(input, belongs_to_type, app_id, user_id).await?;

	Ok(FileRegisterOutput {
		file_id,
		session_id,
	})
}

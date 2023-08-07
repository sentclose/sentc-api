use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::file::{BelongsToType, FileRegisterInput, FileRegisterOutput};
use sentc_crypto_common::{AppId, FileId, GroupId};
use server_api_common::group::group_entities::InternalGroupDataComplete;

use crate::file_entities::FileMetaData;
use crate::{file_model, ApiErrorCodes};

//same values as in file entity
pub(super) const FILE_BELONGS_TO_TYPE_NONE: i32 = 0;
pub(super) const FILE_BELONGS_TO_TYPE_GROUP: i32 = 1;
pub(super) const FILE_BELONGS_TO_TYPE_USER: i32 = 2;

pub async fn register_file(input: FileRegisterInput, app_id: &str, user_id: &str, group_id: Option<GroupId>) -> AppRes<FileRegisterOutput>
{
	//check first if belongs to is set

	let (belongs_to_type, belongs_to) = match input.belongs_to_type {
		BelongsToType::None => (FILE_BELONGS_TO_TYPE_NONE, None),
		BelongsToType::Group => {
			//check if the user got access to this group
			match group_id {
				None => (FILE_BELONGS_TO_TYPE_NONE, None),
				Some(id) => (FILE_BELONGS_TO_TYPE_GROUP, Some(id)),
			}
		},
		BelongsToType::User => {
			//check if the other user is in this app
			match &input.belongs_to_id {
				None => (FILE_BELONGS_TO_TYPE_NONE, None),
				Some(id) => {
					let check = server_api_common::user::check_user_in_app_by_user_id(app_id, id).await?;

					if !check {
						return Err(ServerCoreError::new_msg(
							400,
							ApiErrorCodes::FileNotFound,
							"User not found",
						));
					}

					(FILE_BELONGS_TO_TYPE_USER, Some(id.clone()))
				},
			}
		},
	};

	let (file_id, session_id) = file_model::register_file(
		input.encrypted_key,
		input.encrypted_key_alg,
		input.master_key_id,
		input.encrypted_file_name,
		belongs_to,
		belongs_to_type,
		app_id,
		user_id,
	)
	.await?;

	Ok(FileRegisterOutput {
		file_id,
		session_id,
	})
}

pub async fn get_file(app_id: &str, user_id: Option<&str>, file_id: &str, group_id: Option<&str>) -> AppRes<FileMetaData>
{
	let mut file = file_model::get_file(app_id, file_id).await?;

	match &file.belongs_to_type {
		BelongsToType::None => {},
		BelongsToType::Group => {
			//check if the user got access to this group
			match &file.belongs_to {
				//no group id set for register file
				None => {},
				//check group access
				Some(id) => {
					match group_id {
						None => {
							//user tries to access the file outside of the group routes
							return Err(ServerCoreError::new_msg(
								400,
								ApiErrorCodes::FileAccess,
								"No access to this file",
							));
						},
						Some(g_id) => {
							//user tires to access the file from another group (where he got access in this group)
							if g_id != id {
								return Err(ServerCoreError::new_msg(
									400,
									ApiErrorCodes::FileAccess,
									"No access to this file",
								));
							}
						},
					}
				},
			}
		},
		BelongsToType::User => {
			//check if this user is the actual user
			match &file.belongs_to {
				None => {},
				//user id was set in the file for belongs to
				Some(id) => {
					match user_id {
						//no valid jwt to get the user id
						None => {
							return Err(ServerCoreError::new_msg(
								400,
								ApiErrorCodes::FileAccess,
								"No access to this file",
							));
						},
						Some(user_id) => {
							//valid jwt but user got no access
							if *user_id != *id && user_id != file.owner {
								return Err(ServerCoreError::new_msg(
									400,
									ApiErrorCodes::FileAccess,
									"No access to this file",
								));
							}
						},
					}
				},
			};
		},
	}

	//first page of the part list
	let file_parts = file_model::get_file_parts(app_id, file_id, 0).await?;

	file.part_list = file_parts;

	Ok(file)
}

pub async fn update_file_name(app_id: &str, user_id: &str, file_id: &str, file_name: Option<String>) -> AppRes<()>
{
	let file = file_model::get_file(app_id, file_id).await?;

	//just check for write access, if owner == user id

	if user_id != file.owner {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::FileAccess,
			"No access to this file",
		));
	}

	file_model::update_file_name(file_name, app_id, file_id).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub async fn delete_file(file_id: impl Into<FileId>, app_id: impl Into<AppId>, user_id: &str, group: Option<&InternalGroupDataComplete>)
	-> AppRes<()>
{
	let app_id = app_id.into();
	let file_id = file_id.into();

	let file = file_model::get_file(&app_id, &file_id).await?;

	if file.owner != *user_id {
		match file.belongs_to_type {
			//just check if the user the file owner
			BelongsToType::None => {
				return Err(ServerCoreError::new_msg(
					400,
					ApiErrorCodes::FileAccess,
					"No access to this file",
				));
			},
			BelongsToType::User => {
				return Err(ServerCoreError::new_msg(
					400,
					ApiErrorCodes::FileAccess,
					"No access to this file",
				));
			},
			BelongsToType::Group => {
				//check the group rank, rank <= 3
				match group {
					None => {
						//user tries to access the file outside of the group routes
						return Err(ServerCoreError::new_msg(
							400,
							ApiErrorCodes::FileAccess,
							"No access to this file",
						));
					},
					Some(g) => {
						if g.user_data.rank > 3 {
							return Err(ServerCoreError::new_msg(
								400,
								ApiErrorCodes::FileAccess,
								"No access to this file",
							));
						}
					},
				}
			},
		}
	}

	file_model::delete_file(app_id, file_id).await?;

	Ok(())
}

use std::future::Future;

use sentc_crypto_common::file::{BelongsToType, FileRegisterInput, FileRegisterOutput};
use sentc_crypto_common::{AppId, FileId, GroupId, UserId};

use crate::file::file_entities::FileMetaData;
use crate::file::file_model;
use crate::group::group_entities::InternalGroupDataComplete;
use crate::user::user_service;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

//same values as in file entity
pub(super) static FILE_BELONGS_TO_TYPE_NONE: i32 = 0;
pub(super) static FILE_BELONGS_TO_TYPE_GROUP: i32 = 1;
pub(super) static FILE_BELONGS_TO_TYPE_USER: i32 = 2;

pub async fn register_file(input: FileRegisterInput, app_id: AppId, user_id: UserId, group_id: Option<GroupId>) -> AppRes<FileRegisterOutput>
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
					let check = user_service::check_user_in_app_by_user_id(app_id.to_string(), id.to_string()).await?;

					if !check {
						return Err(HttpErr::new(
							400,
							ApiErrorCodes::FileNotFound,
							"User not found".to_string(),
							None,
						));
					}

					(FILE_BELONGS_TO_TYPE_USER, Some(id.to_string()))
				},
			}
		},
	};

	let (file_id, session_id) = file_model::register_file(
		input.key_id,
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

pub async fn get_file(app_id: AppId, user_id: Option<UserId>, file_id: FileId, group_id: Option<GroupId>) -> AppRes<FileMetaData>
{
	let mut file = file_model::get_file(app_id.to_string(), file_id.to_string()).await?;

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
							return Err(HttpErr::new(
								400,
								ApiErrorCodes::FileAccess,
								"No access to this file".to_string(),
								None,
							));
						},
						Some(g_id) => {
							//user tires to access the file from another group (where he got access in this group)
							if g_id.as_str() != id {
								return Err(HttpErr::new(
									400,
									ApiErrorCodes::FileAccess,
									"No access to this file".to_string(),
									None,
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
							return Err(HttpErr::new(
								400,
								ApiErrorCodes::FileAccess,
								"No access to this file".to_string(),
								None,
							));
						},
						Some(user_id) => {
							//valid jwt but user got no access
							if user_id != id.to_string() && user_id != file.owner {
								return Err(HttpErr::new(
									400,
									ApiErrorCodes::FileAccess,
									"No access to this file".to_string(),
									None,
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

pub async fn update_file_name(app_id: AppId, user_id: UserId, file_id: FileId, file_name: Option<String>) -> AppRes<()>
{
	let file = file_model::get_file(app_id.to_string(), file_id.to_string()).await?;

	//just check for write access, if owner == user id

	if user_id != file.owner {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::FileAccess,
			"No access to this file".to_string(),
			None,
		));
	}

	file_model::update_file_name(file_name, app_id, file_id).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub async fn delete_file(file_id: &str, app_id: &str, user_id: UserId, group: Option<&InternalGroupDataComplete>) -> AppRes<()>
{
	let file = file_model::get_file(app_id.to_string(), file_id.to_string()).await?;

	if file.owner != user_id {
		match file.belongs_to_type {
			//just check if the user the file owner
			BelongsToType::None => {
				return Err(HttpErr::new(
					400,
					ApiErrorCodes::FileAccess,
					"No access to this file".to_string(),
					None,
				));
			},
			BelongsToType::User => {
				return Err(HttpErr::new(
					400,
					ApiErrorCodes::FileAccess,
					"No access to this file".to_string(),
					None,
				));
			},
			BelongsToType::Group => {
				//check the group rank, rank <= 3
				match group {
					None => {
						//user tries to access the file outside of the group routes
						return Err(HttpErr::new(
							400,
							ApiErrorCodes::FileAccess,
							"No access to this file".to_string(),
							None,
						));
					},
					Some(g) => {
						if g.user_data.rank > 3 {
							return Err(HttpErr::new(
								400,
								ApiErrorCodes::FileAccess,
								"No access to this file".to_string(),
								None,
							));
						}
					},
				}
			},
		}
	}

	file_model::delete_file(app_id.to_string(), file_id.to_string()).await?;

	Ok(())
}

pub fn delete_file_for_customer(customer_id: &str) -> impl Future<Output = AppRes<()>>
{
	file_model::delete_files_for_customer(customer_id.to_string())
}

pub fn delete_file_for_app(app_id: &str) -> impl Future<Output = AppRes<()>>
{
	file_model::delete_files_for_app(app_id.to_string())
}

pub fn delete_file_for_group(app_id: &str, group_id: &str, children: Vec<String>) -> impl Future<Output = AppRes<()>>
{
	file_model::delete_files_for_group(app_id.to_string(), group_id.to_string(), children)
}

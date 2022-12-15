use sentc_crypto_common::file::BelongsToType;
use sentc_crypto_common::{AppId, FileId, PartId, SymKeyId, UserId};
use serde::Serialize;
use server_core::take_or_err;

pub struct FileSessionCheck
{
	pub file_id: FileId,
	pub created_at: u128,
	pub max_chunk_size: usize,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for FileSessionCheck
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			file_id: take_or_err!(row, 0, String),
			created_at: take_or_err!(row, 1, u128),
			max_chunk_size: take_or_err!(row, 2, usize),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for FileSessionCheck
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let max_chunk_size: String = take_or_err!(row, 2);
		let max_chunk_size: usize = max_chunk_size.parse().map_err(|e| {
			server_core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			file_id: take_or_err!(row, 0),
			created_at: server_core::take_or_err_u128!(row, 1),
			max_chunk_size,
		})
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize)]
pub struct FileMetaData
{
	pub file_id: FileId,
	pub master_key_id: String,
	pub owner: UserId,
	pub belongs_to: Option<String>,
	pub belongs_to_type: BelongsToType,
	pub key_id: SymKeyId,
	pub time: u128,
	pub encrypted_file_name: Option<String>,
	pub part_list: Vec<FilePartListItem>,
}

impl Into<sentc_crypto_common::file::FileData> for FileMetaData
{
	fn into(self) -> sentc_crypto_common::file::FileData
	{
		let mut part_list: Vec<sentc_crypto_common::file::FilePartListItem> = Vec::with_capacity(self.part_list.len());

		for part in self.part_list {
			part_list.push(part.into());
		}

		sentc_crypto_common::file::FileData {
			file_id: self.file_id,
			master_key_id: self.master_key_id,
			owner: self.owner,
			belongs_to: self.belongs_to,
			belongs_to_type: self.belongs_to_type,
			key_id: self.key_id,
			encrypted_file_name: self.encrypted_file_name,
			part_list,
		}
	}
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for FileMetaData
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		let belongs_to_type = take_or_err!(row, 3, i32);
		let belongs_to_type = match belongs_to_type {
			0 => BelongsToType::None,
			1 => BelongsToType::Group,
			2 => BelongsToType::User,
			_ => BelongsToType::None,
		};

		Ok(Self {
			file_id: take_or_err!(row, 0, String),
			owner: take_or_err!(row, 1, String),
			belongs_to: server_core::take_or_err_opt!(row, 2, String),
			belongs_to_type,
			key_id: take_or_err!(row, 4, String),
			time: take_or_err!(row, 5, u128),
			part_list: Vec::new(),
			encrypted_file_name: server_core::take_or_err_opt!(row, 6, String),
			master_key_id: take_or_err!(row, 7, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for FileMetaData
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let belongs_to_type: i32 = take_or_err!(row, 3);
		let belongs_to_type = match belongs_to_type {
			0 => BelongsToType::None,
			1 => BelongsToType::Group,
			2 => BelongsToType::User,
			_ => BelongsToType::None,
		};

		Ok(Self {
			file_id: take_or_err!(row, 0),
			owner: take_or_err!(row, 1),
			belongs_to: take_or_err!(row, 2),
			belongs_to_type,
			key_id: take_or_err!(row, 4),
			time: server_core::take_or_err_u128!(row, 5),
			encrypted_file_name: take_or_err!(row, 6),
			part_list: Vec::new(),
			master_key_id: take_or_err!(row, 7),
		})
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize)]
pub struct FilePartListItem
{
	pub part_id: PartId,
	pub sequence: i32,
	pub extern_storage: bool,
}

impl Into<sentc_crypto_common::file::FilePartListItem> for FilePartListItem
{
	fn into(self) -> sentc_crypto_common::file::FilePartListItem
	{
		sentc_crypto_common::file::FilePartListItem {
			part_id: self.part_id,
			sequence: self.sequence,
			extern_storage: self.extern_storage,
		}
	}
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for FilePartListItem
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			part_id: take_or_err!(row, 0, String),
			sequence: take_or_err!(row, 1, i32),
			extern_storage: take_or_err!(row, 2, bool),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for FilePartListItem
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			part_id: take_or_err!(row, 0),
			sequence: take_or_err!(row, 1),
			extern_storage: take_or_err!(row, 2),
		})
	}
}

//__________________________________________________________________________________________________

pub struct FilePartListItemDelete
{
	pub part_id: PartId,
	pub sequence: i32,
	pub extern_storage: bool,
	pub app_id: AppId,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for FilePartListItemDelete
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			part_id: take_or_err!(row, 0, String),
			sequence: take_or_err!(row, 1, i32),
			extern_storage: take_or_err!(row, 2, bool),
			app_id: take_or_err!(row, 3, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for FilePartListItemDelete
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			part_id: take_or_err!(row, 0),
			sequence: take_or_err!(row, 1),
			extern_storage: take_or_err!(row, 2),
			app_id: take_or_err!(row, 3),
		})
	}
}

//__________________________________________________________________________________________________

pub struct FileExternalStorageUrl
{
	pub storage_url: String,
	pub app_id: AppId,
	pub auth_token: Option<String>,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for FileExternalStorageUrl
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			storage_url: take_or_err!(row, 0, String),
			app_id: take_or_err!(row, 1, String),
			auth_token: server_core::take_or_err_opt!(row, 2, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for FileExternalStorageUrl
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			storage_url: take_or_err!(row, 0),
			app_id: take_or_err!(row, 1),
			auth_token: take_or_err!(row, 2),
		})
	}
}

//__________________________________________________________________________________________________

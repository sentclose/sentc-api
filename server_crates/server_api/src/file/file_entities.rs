use sentc_crypto_common::FileId;
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
		let created_at: String = take_or_err!(row, 3);
		let created_at: u128 = created_at.parse().map_err(|e| {
			server_core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		let max_chunk_size: String = take_or_err!(row, 2);
		let max_chunk_size: usize = max_chunk_size.parse().map_err(|e| {
			server_core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			file_id: take_or_err!(row, 0),
			created_at,
			max_chunk_size,
		})
	}
}

//__________________________________________________________________________________________________

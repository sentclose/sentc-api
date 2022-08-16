use sentc_crypto_common::crypto::GeneratedSymKeyHeadServerOutput;
use sentc_crypto_common::SymKeyId;
use serde::Serialize;
use server_core::take_or_err;

#[derive(Serialize)]
pub struct SymKeyEntity
{
	key_id: SymKeyId,
	master_key_id: SymKeyId,
	encrypted_key_string: String,
	alg: String,
	time: u128,
}

impl Into<GeneratedSymKeyHeadServerOutput> for SymKeyEntity
{
	fn into(self) -> GeneratedSymKeyHeadServerOutput
	{
		GeneratedSymKeyHeadServerOutput {
			alg: self.alg,
			encrypted_key_string: self.encrypted_key_string,
			master_key_id: self.master_key_id,
			key_id: self.key_id,
			time: self.time,
		}
	}
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for SymKeyEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			key_id: take_or_err!(row, 0, String),
			master_key_id: take_or_err!(row, 1, String),
			encrypted_key_string: take_or_err!(row, 2, String),
			alg: take_or_err!(row, 3, String),
			time: take_or_err!(row, 4, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for SymKeyEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let time: String = take_or_err!(row, 4);
		let time: u128 = time.parse().map_err(|e| {
			server_core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			key_id: take_or_err!(row, 0),
			master_key_id: take_or_err!(row, 1),
			encrypted_key_string: take_or_err!(row, 2),
			alg: take_or_err!(row, 3),
			time,
		})
	}
}

//__________________________________________________________________________________________________

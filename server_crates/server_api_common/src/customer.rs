use sentc_crypto_common::user::{DoneLoginLightOutput, ResetPasswordData, UserDeviceRegisterInput};
use sentc_crypto_common::{AppId, CustomerId};
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use server_core::take_or_err;

#[derive(Serialize, Deserialize)]
pub struct CustomerRegisterData
{
	pub email: String,
	pub register_data: UserDeviceRegisterInput,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerRegisterOutput
{
	pub customer_id: CustomerId,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerDoneRegistrationInput
{
	pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerDoneLoginOutput
{
	pub user_keys: DoneLoginLightOutput,
	pub email_data: CustomerEmailData,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerEmailData
{
	pub validate_email: bool,
	pub email: String,
	pub email_send: u128,
	pub email_status: i32,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerUpdateInput
{
	pub new_email: String,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerResetPasswordInput
{
	pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerDonePasswordResetInput
{
	pub token: String,
	pub reset_password_data: ResetPasswordData,
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct CustomerAppList
{
	pub id: AppId,
	pub identifier: String,
	pub time: u128,
}

#[cfg(feature = "server")]
#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for CustomerAppList
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			id: take_or_err!(row, 0, String),
			identifier: take_or_err!(row, 1, String),
			time: take_or_err!(row, 2, u128),
		})
	}
}

#[cfg(feature = "server")]
#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for CustomerAppList
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let time: String = take_or_err!(row, 2);
		let time: u128 = time.parse().map_err(|e| {
			server_core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			id: take_or_err!(row, 0),
			identifier: take_or_err!(row, 1),
			time,
		})
	}
}

use sentc_crypto_common::user::{CaptchaInput, DoneLoginLightOutput, ResetPasswordData, UserDeviceRegisterInput};
use sentc_crypto_common::{AppId, CustomerId, GroupId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CustomerData
{
	pub name: String,
	pub first_name: String,
	pub company: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerRegisterData
{
	pub customer_data: CustomerData,

	pub email: String,
	pub register_data: UserDeviceRegisterInput,
	pub captcha_input: CaptchaInput,
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
	pub email_data: CustomerDataOutput,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerDataOutput
{
	pub validate_email: bool,
	pub email: String,
	pub email_send: u128,
	pub email_status: i32,
	pub name: String,
	pub first_name: String,
	pub company: Option<String>,
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
	pub captcha_input: CaptchaInput,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerDonePasswordResetInput
{
	pub token: String,
	pub reset_password_data: ResetPasswordData,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerGroupCreateInput
{
	pub name: Option<String>,
	pub des: Option<String>,
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct CustomerAppList
{
	pub id: AppId,
	pub identifier: String,
	pub time: u128,
	pub group_name: Option<String>,
}

#[cfg(feature = "server")]
#[cfg(feature = "mysql")]
impl server_core::db::mysql_async_export::prelude::FromRow for CustomerAppList
{
	fn from_row_opt(mut row: server_core::db::mysql_async_export::Row) -> Result<Self, server_core::db::mysql_async_export::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			id: server_core::take_or_err!(row, 0, String),
			identifier: server_core::take_or_err!(row, 1, String),
			time: server_core::take_or_err!(row, 2, u128),
			group_name: server_core::take_or_err_opt!(row, 3, String),
		})
	}
}

#[cfg(feature = "server")]
#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for CustomerAppList
{
	fn from_row_opt(row: &server_core::db::rusqlite_export::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			id: server_core::take_or_err!(row, 0),
			identifier: server_core::take_or_err!(row, 1),
			time: server_core::take_or_err_u128!(row, 2),
			group_name: server_core::take_or_err!(row, 3),
		})
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct CustomerGroupView
{
	pub data: CustomerGroupList,
	pub apps: Vec<CustomerAppList>,
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct CustomerGroupList
{
	pub id: GroupId,
	pub time: u128,
	pub rank: i32,
	pub group_name: Option<String>,
	pub des: Option<String>,
}

#[cfg(feature = "server")]
#[cfg(feature = "mysql")]
impl server_core::db::mysql_async_export::prelude::FromRow for CustomerGroupList
{
	fn from_row_opt(mut row: server_core::db::mysql_async_export::Row) -> Result<Self, server_core::db::mysql_async_export::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			id: server_core::take_or_err!(row, 0, String),
			time: server_core::take_or_err!(row, 1, u128),
			rank: server_core::take_or_err!(row, 2, i32),
			group_name: server_core::take_or_err_opt!(row, 3, String),
			des: server_core::take_or_err_opt!(row, 4, String),
		})
	}
}

#[cfg(feature = "server")]
#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for CustomerGroupList
{
	fn from_row_opt(row: &server_core::db::rusqlite_export::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			id: server_core::take_or_err!(row, 0),
			time: server_core::take_or_err_u128!(row, 1),
			rank: server_core::take_or_err!(row, 2),
			group_name: server_core::take_or_err_opt!(row, 3),
			des: server_core::take_or_err_opt!(row, 4),
		})
	}
}

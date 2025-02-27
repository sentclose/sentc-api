use sentc_crypto_common::group::GroupUserListItem;
use sentc_crypto_common::user::{CaptchaInput, UserDeviceRegisterInput, VerifyLoginLightOutput};
use sentc_crypto_common::{AppId, CustomerId, GroupId, UserId};
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
	pub verify: VerifyLoginLightOutput,
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
	pub reset_password_data: UserDeviceRegisterInput,
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
	pub disabled: Option<i32>,
	pub disabled_ts: Option<u128>,
}

#[cfg(feature = "server")]
#[cfg(feature = "mysql")]
impl rustgram_server_util::db::mysql_async_export::prelude::FromRow for CustomerAppList
{
	fn from_row_opt(
		mut row: rustgram_server_util::db::mysql_async_export::Row,
	) -> Result<Self, rustgram_server_util::db::mysql_async_export::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			id: rustgram_server_util::take_or_err!(row, 0, String),
			identifier: rustgram_server_util::take_or_err!(row, 1, String),
			time: rustgram_server_util::take_or_err!(row, 2, u128),
			group_name: rustgram_server_util::take_or_err_opt!(row, 3, String),
			disabled: rustgram_server_util::take_or_err_opt!(row, 4, i32),
			disabled_ts: rustgram_server_util::take_or_err_opt!(row, 5, u128),
		})
	}
}

#[cfg(feature = "server")]
#[cfg(feature = "sqlite")]
impl rustgram_server_util::db::FromSqliteRow for CustomerAppList
{
	fn from_row_opt(row: &rustgram_server_util::db::rusqlite_export::Row) -> Result<Self, rustgram_server_util::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let disabled_ts: Option<String> = rustgram_server_util::take_or_err!(row, 5);

		let disabled_ts = if let Some(d) = disabled_ts {
			match d.parse() {
				Ok(v) => Some(v),
				Err(e) => {
					return Err(rustgram_server_util::db::FormSqliteRowError {
						msg: format!("err in db fetch: {:?}", e),
					})
				},
			}
		} else {
			None
		};

		Ok(Self {
			id: rustgram_server_util::take_or_err!(row, 0),
			identifier: rustgram_server_util::take_or_err!(row, 1),
			time: rustgram_server_util::take_or_err_u128!(row, 2),
			group_name: rustgram_server_util::take_or_err!(row, 3),
			disabled: rustgram_server_util::take_or_err!(row, 4),
			disabled_ts,
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
#[cfg_attr(feature = "server", derive(rustgram_server_util::DB))]
pub struct CustomerGroupList
{
	pub id: GroupId,
	pub time: u128,
	pub rank: i32,
	pub group_name: Option<String>,
	pub des: Option<String>,
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct CustomerList
{
	pub id: UserId,
	pub first_name: String,
	pub name: String,
	pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerGroupMemberFetch
{
	pub group_member: Vec<GroupUserListItem>,
	pub customer_data: Vec<CustomerList>,
}

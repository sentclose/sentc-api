#![allow(dead_code)]

use sentc_crypto_common::{CustomerId, DeviceId, UserId};
use serde::{Deserialize, Serialize};

use crate::sentc_group_entities::GroupUserListItem;

#[cfg(feature = "send_mail")]
pub(crate) enum RegisterEmailStatus
{
	Success,
	FailedMessage(String),
	FailedSend(String),
	Other(String),
}

//__________________________________________________________________________________________________

#[cfg_attr(feature = "mysql", derive(rustgram_server_util::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(rustgram_server_util::Sqlite))]
pub(crate) struct CustomerDataEntity
{
	pub email: String,
	pub email_valid: i32,
	pub email_send: u128,
	pub email_status: i32,
	pub company: Option<String>,
	pub first_name: String,
	pub name: String,
}

impl Into<server_api_common::customer::CustomerDataOutput> for CustomerDataEntity
{
	fn into(self) -> server_api_common::customer::CustomerDataOutput
	{
		let validate_email = match self.email_valid {
			0 => false,
			1 => true,
			_ => false,
		};

		server_api_common::customer::CustomerDataOutput {
			validate_email,
			email: self.email,
			email_send: self.email_send,
			email_status: self.email_status,
			name: self.name,
			first_name: self.first_name,
			company: self.company,
		}
	}
}

//__________________________________________________________________________________________________

#[cfg_attr(feature = "mysql", derive(rustgram_server_util::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(rustgram_server_util::Sqlite))]
pub(crate) struct CustomerDataByEmailEntity
{
	pub id: CustomerId,
	pub email_valid: i32,
	pub email_send: u128,
	pub email_status: i32,
}

//__________________________________________________________________________________________________

#[cfg_attr(feature = "mysql", derive(rustgram_server_util::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(rustgram_server_util::Sqlite))]
pub(crate) struct CustomerEmailToken
{
	pub email_token: String,
	pub email: String,
}

//__________________________________________________________________________________________________

#[cfg_attr(feature = "mysql", derive(rustgram_server_util::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(rustgram_server_util::Sqlite))]
pub(crate) struct CustomerEmailByToken
{
	pub email: String,
	pub id: CustomerId,
	pub device_id: DeviceId,
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "mysql", derive(rustgram_server_util::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(rustgram_server_util::Sqlite))]
pub struct CustomerList
{
	pub id: UserId,
	pub first_name: String,
	pub name: String,
	pub email: String,
}

impl Into<server_api_common::customer::CustomerList> for CustomerList
{
	fn into(self) -> server_api_common::customer::CustomerList
	{
		server_api_common::customer::CustomerList {
			id: self.id,
			first_name: self.first_name,
			name: self.name,
			email: self.email,
		}
	}
}

#[derive(Serialize)]
pub struct CustomerGroupMemberFetch
{
	pub group_member: Vec<GroupUserListItem>,
	pub customer_data: Vec<CustomerList>,
}

impl Into<server_api_common::customer::CustomerGroupMemberFetch> for CustomerGroupMemberFetch
{
	fn into(self) -> server_api_common::customer::CustomerGroupMemberFetch
	{
		server_api_common::customer::CustomerGroupMemberFetch {
			group_member: self.group_member.into_iter().map(|i| i.into()).collect(),
			customer_data: self.customer_data.into_iter().map(|i| i.into()).collect(),
		}
	}
}

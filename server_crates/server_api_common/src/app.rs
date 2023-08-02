use sentc_crypto_common::{AppId, JwtKeyId, SignKeyPairId};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::customer::CustomerAppList;

/**
The options to control the access to the api.

0 = not allowed
1 = with public or secret token
2 = only with secret token
*/
#[derive(Serialize, Deserialize)]
pub struct AppOptions
{
	pub group_create: i32,
	pub group_get: i32,

	pub group_user_keys: i32,
	pub group_user_update_check: i32,

	pub group_invite: i32,
	pub group_auto_invite: i32,
	pub group_reject_invite: i32,
	pub group_accept_invite: i32,

	pub group_join_req: i32,
	pub group_accept_join_req: i32,
	pub group_reject_join_req: i32,

	pub group_key_rotation: i32,

	pub group_user_delete: i32,
	pub group_delete: i32,

	pub group_leave: i32,
	pub group_change_rank: i32,

	pub user_exists: i32,
	pub user_register: i32,
	pub user_delete: i32,
	pub user_update: i32,
	pub user_change_password: i32,
	pub user_reset_password: i32,
	pub user_prepare_login: i32,
	pub user_done_login: i32,
	pub user_public_data: i32,
	pub user_jwt_refresh: i32,

	pub key_register: i32,
	pub key_get: i32,

	pub group_list: i32,

	pub file_register: i32,
	pub file_part_upload: i32,
	pub file_get: i32,
	pub file_part_download: i32,
	pub file_delete: i32,

	pub user_device_register: i32,
	pub user_device_delete: i32,
	pub user_device_list: i32,

	pub group_invite_stop: i32,

	pub user_key_update: i32,

	pub content: i32,

	pub content_small: i32,
	pub content_med: i32,
	pub content_large: i32,
	pub content_x_large: i32,

	pub user_register_otp: i32,
	pub user_reset_otp: i32,
	pub user_disable_otp: i32,
	pub user_get_otp_recovery_keys: i32,
}

impl Default for AppOptions
{
	fn default() -> Self
	{
		Self {
			group_create: 1,
			group_get: 1,
			group_user_keys: 1,
			group_user_update_check: 1,
			group_invite: 1,
			group_auto_invite: 1,
			group_reject_invite: 1,
			group_accept_invite: 1,
			group_join_req: 1,
			group_accept_join_req: 1,
			group_reject_join_req: 1,
			group_key_rotation: 1,
			group_user_delete: 1,
			group_delete: 1,
			group_leave: 1,
			group_change_rank: 1,
			user_exists: 1,
			user_register: 2,
			user_delete: 2,
			user_update: 1,
			user_change_password: 1,
			user_reset_password: 1,
			user_prepare_login: 1,
			user_done_login: 1,
			user_public_data: 1,
			user_jwt_refresh: 1,
			key_register: 1,
			key_get: 1,
			group_list: 1,
			file_register: 1,
			file_part_upload: 1,
			file_get: 1,
			file_part_download: 1,
			file_delete: 1,
			user_device_register: 1,
			user_device_delete: 1,
			user_device_list: 1,
			group_invite_stop: 1,
			user_key_update: 1,
			content: 1,
			content_small: 2,
			content_med: 2,
			content_large: 2,
			content_x_large: 2,
			user_register_otp: 1,
			user_reset_otp: 1,
			user_disable_otp: 1,
			user_get_otp_recovery_keys: 1,
		}
	}
}

impl AppOptions
{
	pub fn default_closed() -> Self
	{
		Self {
			group_create: 0,
			group_get: 0,
			group_user_keys: 0,
			group_user_update_check: 0,
			group_invite: 0,
			group_auto_invite: 0,
			group_reject_invite: 0,
			group_accept_invite: 0,
			group_join_req: 0,
			group_accept_join_req: 0,
			group_reject_join_req: 0,
			group_key_rotation: 0,
			group_user_delete: 0,
			group_delete: 0,
			group_leave: 0,
			group_change_rank: 0,
			user_exists: 0,
			user_register: 0,
			user_delete: 0,
			user_update: 0,
			user_change_password: 0,
			user_reset_password: 0,
			user_prepare_login: 0,
			user_done_login: 0,
			user_public_data: 0,
			user_jwt_refresh: 0,
			key_register: 0,
			key_get: 0,
			group_list: 0,
			file_register: 0,
			file_part_upload: 0,
			file_get: 0,
			file_part_download: 0,
			file_delete: 0,
			user_device_register: 0,
			user_device_delete: 0,
			user_device_list: 0,
			group_invite_stop: 0,
			user_key_update: 0,
			content: 0,
			content_small: 0,
			content_med: 0,
			content_large: 0,
			content_x_large: 0,
			user_register_otp: 0,
			user_reset_otp: 0,
			user_disable_otp: 0,
			user_get_otp_recovery_keys: 0,
		}
	}

	pub fn default_lax() -> Self
	{
		Self {
			group_create: 1,
			group_get: 1,
			group_user_keys: 1,
			group_user_update_check: 1,
			group_invite: 1,
			group_auto_invite: 1,
			group_reject_invite: 1,
			group_accept_invite: 1,
			group_join_req: 1,
			group_accept_join_req: 1,
			group_reject_join_req: 1,
			group_key_rotation: 1,
			group_user_delete: 1,
			group_delete: 1,
			group_leave: 1,
			group_change_rank: 1,
			user_exists: 1,
			user_register: 1,
			user_delete: 1,
			user_update: 1,
			user_change_password: 1,
			user_reset_password: 1,
			user_prepare_login: 1,
			user_done_login: 1,
			user_public_data: 1,
			user_jwt_refresh: 1,
			key_register: 1,
			key_get: 1,
			group_list: 1,
			file_register: 1,
			file_part_upload: 1,
			file_get: 1,
			file_part_download: 1,
			file_delete: 1,
			user_device_register: 1,
			user_device_delete: 1,
			user_device_list: 1,
			group_invite_stop: 1,
			user_key_update: 1,
			content: 1,
			content_small: 1,
			content_med: 1,
			content_large: 1,
			content_x_large: 1,
			user_register_otp: 1,
			user_reset_otp: 1,
			user_disable_otp: 1,
			user_get_otp_recovery_keys: 1,
		}
	}
}

#[cfg(feature = "server")]
#[cfg(feature = "mysql")]
impl rustgram_server_util::db::mysql_async_export::prelude::FromRow for AppOptions
{
	fn from_row_opt(
		mut row: rustgram_server_util::db::mysql_async_export::Row,
	) -> Result<Self, rustgram_server_util::db::mysql_async_export::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			group_create: rustgram_server_util::take_or_err!(row, 0, i32),
			group_get: rustgram_server_util::take_or_err!(row, 1, i32),
			group_user_keys: rustgram_server_util::take_or_err!(row, 2, i32),
			group_user_update_check: rustgram_server_util::take_or_err!(row, 3, i32),

			group_invite: rustgram_server_util::take_or_err!(row, 4, i32),
			group_reject_invite: rustgram_server_util::take_or_err!(row, 5, i32),
			group_accept_invite: rustgram_server_util::take_or_err!(row, 6, i32),

			group_join_req: rustgram_server_util::take_or_err!(row, 7, i32),
			group_accept_join_req: rustgram_server_util::take_or_err!(row, 8, i32),
			group_reject_join_req: rustgram_server_util::take_or_err!(row, 9, i32),

			group_key_rotation: rustgram_server_util::take_or_err!(row, 10, i32),

			group_user_delete: rustgram_server_util::take_or_err!(row, 11, i32),

			group_delete: rustgram_server_util::take_or_err!(row, 12, i32),

			group_leave: rustgram_server_util::take_or_err!(row, 13, i32),
			group_change_rank: rustgram_server_util::take_or_err!(row, 14, i32),

			user_exists: rustgram_server_util::take_or_err!(row, 15, i32),
			user_register: rustgram_server_util::take_or_err!(row, 16, i32),
			user_delete: rustgram_server_util::take_or_err!(row, 17, i32),
			user_update: rustgram_server_util::take_or_err!(row, 18, i32),
			user_change_password: rustgram_server_util::take_or_err!(row, 19, i32),
			user_reset_password: rustgram_server_util::take_or_err!(row, 20, i32),
			user_prepare_login: rustgram_server_util::take_or_err!(row, 21, i32),
			user_done_login: rustgram_server_util::take_or_err!(row, 22, i32),
			user_public_data: rustgram_server_util::take_or_err!(row, 23, i32),
			user_jwt_refresh: rustgram_server_util::take_or_err!(row, 24, i32),

			key_register: rustgram_server_util::take_or_err!(row, 25, i32),
			key_get: rustgram_server_util::take_or_err!(row, 26, i32),

			group_auto_invite: rustgram_server_util::take_or_err!(row, 27, i32),
			group_list: rustgram_server_util::take_or_err!(row, 28, i32),

			file_register: rustgram_server_util::take_or_err!(row, 29, i32),
			file_part_upload: rustgram_server_util::take_or_err!(row, 30, i32),
			file_get: rustgram_server_util::take_or_err!(row, 31, i32),
			file_part_download: rustgram_server_util::take_or_err!(row, 32, i32),
			user_device_register: rustgram_server_util::take_or_err!(row, 33, i32),
			user_device_delete: rustgram_server_util::take_or_err!(row, 34, i32),
			user_device_list: rustgram_server_util::take_or_err!(row, 35, i32),
			group_invite_stop: rustgram_server_util::take_or_err!(row, 36, i32),

			user_key_update: rustgram_server_util::take_or_err!(row, 37, i32),

			file_delete: rustgram_server_util::take_or_err!(row, 38, i32),
			content: rustgram_server_util::take_or_err!(row, 39, i32),

			content_small: rustgram_server_util::take_or_err!(row, 40, i32),
			content_med: rustgram_server_util::take_or_err!(row, 41, i32),
			content_large: rustgram_server_util::take_or_err!(row, 42, i32),
			content_x_large: rustgram_server_util::take_or_err!(row, 43, i32),

			user_register_otp: rustgram_server_util::take_or_err!(row, 44, i32),
			user_reset_otp: rustgram_server_util::take_or_err!(row, 45, i32),
			user_disable_otp: rustgram_server_util::take_or_err!(row, 46, i32),
			user_get_otp_recovery_keys: rustgram_server_util::take_or_err!(row, 47, i32),
		})
	}
}

#[cfg(feature = "server")]
#[cfg(feature = "sqlite")]
impl rustgram_server_util::db::FromSqliteRow for AppOptions
{
	fn from_row_opt(row: &rustgram_server_util::db::rusqlite_export::Row) -> Result<Self, rustgram_server_util::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			group_create: rustgram_server_util::take_or_err!(row, 0),
			group_get: rustgram_server_util::take_or_err!(row, 1),
			group_user_keys: rustgram_server_util::take_or_err!(row, 2),
			group_user_update_check: rustgram_server_util::take_or_err!(row, 3),

			group_invite: rustgram_server_util::take_or_err!(row, 4),
			group_reject_invite: rustgram_server_util::take_or_err!(row, 5),
			group_accept_invite: rustgram_server_util::take_or_err!(row, 6),

			group_join_req: rustgram_server_util::take_or_err!(row, 7),
			group_accept_join_req: rustgram_server_util::take_or_err!(row, 8),
			group_reject_join_req: rustgram_server_util::take_or_err!(row, 9),

			group_key_rotation: rustgram_server_util::take_or_err!(row, 10),

			group_user_delete: rustgram_server_util::take_or_err!(row, 11),

			group_delete: rustgram_server_util::take_or_err!(row, 12),

			group_leave: rustgram_server_util::take_or_err!(row, 13),
			group_change_rank: rustgram_server_util::take_or_err!(row, 14),

			user_exists: rustgram_server_util::take_or_err!(row, 15),
			user_register: rustgram_server_util::take_or_err!(row, 16),
			user_delete: rustgram_server_util::take_or_err!(row, 17),
			user_update: rustgram_server_util::take_or_err!(row, 18),
			user_change_password: rustgram_server_util::take_or_err!(row, 19),
			user_reset_password: rustgram_server_util::take_or_err!(row, 20),
			user_prepare_login: rustgram_server_util::take_or_err!(row, 21),
			user_done_login: rustgram_server_util::take_or_err!(row, 22),
			user_public_data: rustgram_server_util::take_or_err!(row, 23),
			user_jwt_refresh: rustgram_server_util::take_or_err!(row, 24),

			key_register: rustgram_server_util::take_or_err!(row, 25),
			key_get: rustgram_server_util::take_or_err!(row, 26),

			group_auto_invite: rustgram_server_util::take_or_err!(row, 27),
			group_list: rustgram_server_util::take_or_err!(row, 28),

			file_register: rustgram_server_util::take_or_err!(row, 29),
			file_part_upload: rustgram_server_util::take_or_err!(row, 30),
			file_get: rustgram_server_util::take_or_err!(row, 31),
			file_part_download: rustgram_server_util::take_or_err!(row, 32),
			user_device_register: rustgram_server_util::take_or_err!(row, 33),
			user_device_delete: rustgram_server_util::take_or_err!(row, 34),
			user_device_list: rustgram_server_util::take_or_err!(row, 35),
			group_invite_stop: rustgram_server_util::take_or_err!(row, 36),

			user_key_update: rustgram_server_util::take_or_err!(row, 37),

			file_delete: rustgram_server_util::take_or_err!(row, 38),
			content: rustgram_server_util::take_or_err!(row, 39),

			content_small: rustgram_server_util::take_or_err!(row, 40),
			content_med: rustgram_server_util::take_or_err!(row, 41),
			content_large: rustgram_server_util::take_or_err!(row, 42),
			content_x_large: rustgram_server_util::take_or_err!(row, 43),

			user_register_otp: rustgram_server_util::take_or_err!(row, 44),
			user_reset_otp: rustgram_server_util::take_or_err!(row, 45),
			user_disable_otp: rustgram_server_util::take_or_err!(row, 46),
			user_get_otp_recovery_keys: rustgram_server_util::take_or_err!(row, 47),
		})
	}
}

#[derive(Serialize, Deserialize)]
pub struct AppDetails
{
	pub options: AppOptions,
	pub file_options: AppFileOptionsInput,
	pub group_options: AppGroupOption,
	pub details: CustomerAppList,
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct AppRegisterInput
{
	pub identifier: Option<String>,
	pub options: AppOptions, //if no options then use the defaults
	pub file_options: AppFileOptionsInput,
	pub group_options: AppGroupOption,
}

impl AppRegisterInput
{
	pub fn to_string(&self) -> serde_json::Result<String>
	{
		to_string(self)
	}
}

#[derive(Serialize, Deserialize)]
pub struct AppUpdateInput
{
	pub identifier: Option<String>,
}

impl AppUpdateInput
{
	pub fn to_string(&self) -> serde_json::Result<String>
	{
		to_string(self)
	}
}

/**
When creating multiple jwt keys for this app

Always return this for every new jwt key pair
 */
#[derive(Serialize, Deserialize)]
pub struct AppJwtRegisterOutput
{
	pub app_id: AppId,
	pub jwt_id: JwtKeyId,
	pub jwt_verify_key: String,
	pub jwt_sign_key: String,
	pub jwt_alg: String, //should be ES384 for now
}

#[derive(Serialize, Deserialize)]
pub struct AppRegisterOutput
{
	pub app_id: AppId,

	//don't show this values in te normal app data
	pub secret_token: String,
	pub public_token: String,

	pub jwt_data: AppJwtRegisterOutput,
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct AppTokenRenewOutput
{
	pub secret_token: String,
	pub public_token: String,
}

//__________________________________________________________________________________________________

//copy from app internal entity but without the db trait impl
#[derive(Serialize, Deserialize)]
pub struct AppJwtData
{
	pub jwt_key_id: SignKeyPairId,
	pub jwt_alg: String, //should be ES384 for now
	pub time: u128,
	pub sign_key: String,
	pub verify_key: String,
}

#[cfg(feature = "server")]
#[cfg(feature = "mysql")]
impl rustgram_server_util::db::mysql_async_export::prelude::FromRow for AppJwtData
{
	fn from_row_opt(
		mut row: rustgram_server_util::db::mysql_async_export::Row,
	) -> Result<Self, rustgram_server_util::db::mysql_async_export::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			jwt_key_id: rustgram_server_util::take_or_err!(row, 0, String),
			jwt_alg: rustgram_server_util::take_or_err!(row, 1, String),
			time: rustgram_server_util::take_or_err!(row, 2, u128),
			sign_key: rustgram_server_util::take_or_err!(row, 3, String),
			verify_key: rustgram_server_util::take_or_err!(row, 4, String),
		})
	}
}

#[cfg(feature = "server")]
#[cfg(feature = "sqlite")]
impl rustgram_server_util::db::FromSqliteRow for AppJwtData
{
	fn from_row_opt(row: &rustgram_server_util::db::rusqlite_export::Row) -> Result<Self, rustgram_server_util::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			jwt_key_id: rustgram_server_util::take_or_err!(row, 0),
			jwt_alg: rustgram_server_util::take_or_err!(row, 1),
			time: rustgram_server_util::take_or_err_u128!(row, 2),
			sign_key: rustgram_server_util::take_or_err!(row, 3),
			verify_key: rustgram_server_util::take_or_err!(row, 4),
		})
	}
}

//__________________________________________________________________________________________________

pub const FILE_STORAGE_NONE: i32 = -1;
pub const FILE_STORAGE_SENTC: i32 = 0;
pub const FILE_STORAGE_OWN: i32 = 1;

#[derive(Serialize, Deserialize)]
pub struct AppFileOptionsInput
{
	pub file_storage: i32,
	pub storage_url: Option<String>,
	pub auth_token: Option<String>,
}

impl Default for AppFileOptionsInput
{
	fn default() -> Self
	{
		Self {
			file_storage: FILE_STORAGE_SENTC,
			storage_url: None,
			auth_token: None,
		}
	}
}

impl AppFileOptionsInput
{
	pub fn default_closed() -> Self
	{
		Self {
			file_storage: FILE_STORAGE_NONE,
			storage_url: None,
			auth_token: None,
		}
	}
}

#[cfg(feature = "server")]
#[cfg(feature = "mysql")]
impl rustgram_server_util::db::mysql_async_export::prelude::FromRow for AppFileOptionsInput
{
	fn from_row_opt(
		mut row: rustgram_server_util::db::mysql_async_export::Row,
	) -> Result<Self, rustgram_server_util::db::mysql_async_export::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			file_storage: rustgram_server_util::take_or_err!(row, 0, i32),
			storage_url: rustgram_server_util::take_or_err_opt!(row, 1, String),
			auth_token: rustgram_server_util::take_or_err_opt!(row, 2, String),
		})
	}
}

#[cfg(feature = "server")]
#[cfg(feature = "sqlite")]
impl rustgram_server_util::db::FromSqliteRow for AppFileOptionsInput
{
	fn from_row_opt(row: &rustgram_server_util::db::rusqlite_export::Row) -> Result<Self, rustgram_server_util::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			file_storage: rustgram_server_util::take_or_err!(row, 0),
			storage_url: rustgram_server_util::take_or_err!(row, 1),
			auth_token: rustgram_server_util::take_or_err!(row, 2),
		})
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
pub struct AppGroupOption
{
	pub max_key_rotation_month: i32,
	pub min_rank_key_rotation: i32,
}

impl Default for AppGroupOption
{
	fn default() -> Self
	{
		Self {
			max_key_rotation_month: 100,
			min_rank_key_rotation: 4,
		}
	}
}

#[cfg(feature = "server")]
#[cfg(feature = "mysql")]
impl rustgram_server_util::db::mysql_async_export::prelude::FromRow for AppGroupOption
{
	fn from_row_opt(
		mut row: rustgram_server_util::db::mysql_async_export::Row,
	) -> Result<Self, rustgram_server_util::db::mysql_async_export::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			max_key_rotation_month: rustgram_server_util::take_or_err!(row, 0, i32),
			min_rank_key_rotation: rustgram_server_util::take_or_err!(row, 1, i32),
		})
	}
}

#[cfg(feature = "server")]
#[cfg(feature = "sqlite")]
impl rustgram_server_util::db::FromSqliteRow for AppGroupOption
{
	fn from_row_opt(row: &rustgram_server_util::db::rusqlite_export::Row) -> Result<Self, rustgram_server_util::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			max_key_rotation_month: rustgram_server_util::take_or_err!(row, 0),
			min_rank_key_rotation: rustgram_server_util::take_or_err!(row, 1),
		})
	}
}

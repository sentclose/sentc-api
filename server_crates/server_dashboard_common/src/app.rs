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
#[cfg_attr(feature = "server", derive(rustgram_server_util::DB))]
pub struct AppOptions
{
	pub group_create: i32,
	pub group_get: i32,

	pub group_user_keys: i32,
	pub group_user_update_check: i32,

	pub group_invite: i32,
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

	pub group_auto_invite: i32,
	pub group_list: i32,

	pub file_register: i32,
	pub file_part_upload: i32,
	pub file_get: i32,
	pub file_part_download: i32,

	pub user_device_register: i32,
	pub user_device_delete: i32,
	pub user_device_list: i32,

	pub group_invite_stop: i32,

	pub user_key_update: i32,

	pub file_delete: i32,
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
#[cfg_attr(feature = "server", derive(rustgram_server_util::DB))]
pub struct AppJwtData
{
	pub jwt_key_id: SignKeyPairId,
	pub jwt_alg: String, //should be ES384 for now
	pub time: u128,
	pub sign_key: String,
	pub verify_key: String,
}

//__________________________________________________________________________________________________

pub const FILE_STORAGE_NONE: i32 = -1;
pub const FILE_STORAGE_SENTC: i32 = 0;
pub const FILE_STORAGE_OWN: i32 = 1;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(rustgram_server_util::DB))]
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

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(rustgram_server_util::DB))]
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

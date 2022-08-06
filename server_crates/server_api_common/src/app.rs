use sentc_crypto_common::{AppId, CustomerId, JwtKeyId, SignKeyPairId};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

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
}

impl Default for AppOptions
{
	fn default() -> Self
	{
		Self {
			group_create: 2,
			group_get: 1,
			group_invite: 1,
			group_reject_invite: 1,
			group_accept_invite: 1,
			group_join_req: 1,
			group_accept_join_req: 1,
			group_reject_join_req: 1,
			group_key_rotation: 1,
			group_user_delete: 1,
			group_delete: 2,
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
		}
	}
}

impl AppOptions
{
	pub fn default_lax() -> Self
	{
		Self {
			group_create: 1,
			group_get: 1,
			group_invite: 1,
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
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct AppRegisterInput
{
	pub identifier: Option<String>,
	pub options: AppOptions, //if no options then use the defaults
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
	pub customer_id: CustomerId,
	pub app_id: AppId,
	pub jwt_id: JwtKeyId,
	pub jwt_verify_key: String,
	pub jwt_sign_key: String,
	pub jwt_alg: String, //should be ES384 for now
}

#[derive(Serialize, Deserialize)]
pub struct AppRegisterOutput
{
	pub customer_id: CustomerId,
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

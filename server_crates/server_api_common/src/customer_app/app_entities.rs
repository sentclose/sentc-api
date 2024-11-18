use rustgram_server_util::DB;
use sentc_crypto_common::{AppId, CustomerId, SignKeyPairId};
use serde::{Deserialize, Serialize};
use server_dashboard_common::app::{AppGroupOption, AppOptions};

/**
Data which is used to identify the customers app requests.

Cache this data:
- the valid jwt keys
- the general app data

Only internal values from the db
 */
#[derive(Serialize, Deserialize)]
pub struct AppData
{
	pub app_data: AppDataGeneral,
	pub jwt_data: Vec<AppJwt>, //use the newest jwt data to create a jwt, but use the old one to validate the old jwt.
	pub auth_with_token: AuthWithToken,
	pub options: AppOptions,
	pub file_options: AppFileOptions,
	pub group_options: AppGroupOption,
}

#[derive(Serialize, Deserialize, DB)]
pub struct AppFileOptions
{
	pub file_storage: i32,
	pub storage_url: Option<String>,
}

/**
Describe what token was sent from the req, the public or private
 */
#[derive(Serialize, Deserialize)]
pub enum AuthWithToken
{
	Public,
	Secret,
}

//__________________________________________________________________________________________________

/**
This values can only be exists once

Only internal values from the db
 */
#[derive(Serialize, Deserialize, DB)]
pub struct AppDataGeneral
{
	pub app_id: AppId,
	pub owner_id: CustomerId,
	pub owner_type: i32,
	pub hashed_secret_token: String,
	pub hashed_public_token: String,
	pub hash_alg: String,
	pub disabled: Option<i32>,
}

//__________________________________________________________________________________________________

/**
The key data for creating jwt for the app.

When customer logged in the dashboard, sentc internal keys are used.

It is possible to have multiple valid jwt keys.

Only internal values from the db
 */
#[derive(Serialize, Deserialize, DB)]
pub struct AppJwt
{
	pub jwt_key_id: SignKeyPairId,
	pub jwt_alg: String, //should be ES384 for now
	pub time: u128,
}

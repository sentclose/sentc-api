use sentc_crypto_common::user::RegisterData;
use sentc_crypto_common::{AppId, CustomerId, CustomerPublicTokenId, CustomerSecretTokenId, SignKeyPairId};
use serde::{Deserialize, Serialize};

/**
Data which is used to identify the customers app requests.

Cache this data:
- the valid jwt keys
- the general app data

Only internal values from the db
*/
pub struct CustomerAppData
{
	pub app_data: CustomerAppDataGeneral,
	pub jwt_data: Vec<CustomerAppJwt>, //use the newest jwt data to create a jwt, but use the old one to validate the old jwt.
}

/**
This values can only be exists once

Only internal values from the db
*/
pub struct CustomerAppDataGeneral
{
	pub customer_id: CustomerId,
	pub app_id: AppId,
	pub secret_token_id: CustomerSecretTokenId,
	pub public_token_id: CustomerPublicTokenId,
	pub hashed_secret_token: String,
	pub hashed_public_token: String,
}

/**
The key data for creating jwt for the app.

When customer logged in in the dashboard, sentc internal keys are used.

It is possible to have multiple valid jwt keys.

Only internal values from the db
*/
pub struct CustomerAppJwt
{
	pub jwt_key_id: SignKeyPairId,
	pub jwt_alg: String, //should be ES384 for now
}

#[derive(Serialize, Deserialize)]
pub struct CustomerRegisterData
{
	pub email: String,
	pub register_data: RegisterData,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerRegisterOutput
{
	pub customer_id: String,
}

/**
When creating multiple jwt keys for this app
*/
#[derive(Serialize, Deserialize)]
pub struct CustomerAppJwtRegisterOutput
{
	pub customer_id: CustomerId,
	pub app_id: AppId,
	pub jwt_verify_key: String,
	pub jwt_sign_key: String,
	pub jwt_alg: String, //should be ES384 for now
}

#[derive(Serialize, Deserialize)]
pub struct CustomerAppRegisterOutput
{
	pub customer_id: CustomerId,
	pub app_id: AppId,

	//don't show this values in te normal app data
	pub secret_token: String,
	pub public_token: String,

	pub jwt_data: CustomerAppJwtRegisterOutput,
}

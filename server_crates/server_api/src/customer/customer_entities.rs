use sentc_crypto_common::{AppId, CustomerId, CustomerPublicTokenId, CustomerSecretTokenId, SignKeyPairId};
use serde::{Deserialize, Serialize};

/**
Data which is used to identify the customers app requests.

Cache this data:
- the valid jwt keys
- the general app data
*/
#[derive(Serialize, Deserialize)]
pub struct CustomerAppData
{
	pub app_data: CustomerAppDataGeneral,
	pub jwt_data: Vec<CustomerAppJwt>,
}

/**
This values can only be exists once
*/
#[derive(Serialize, Deserialize)]
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
*/
#[derive(Serialize, Deserialize)]
pub struct CustomerAppJwt
{
	pub jwt_key_id: SignKeyPairId,
	pub jwt_alg: String, //should be ES384 for now
}

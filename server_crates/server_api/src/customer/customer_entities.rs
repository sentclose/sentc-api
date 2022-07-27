use sentc_crypto_common::user::RegisterData;
use sentc_crypto_common::CustomerId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CustomerRegisterData
{
	pub email: String,
	pub register_data: RegisterData,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerRegisterOutput
{
	pub customer_id: CustomerId,
}

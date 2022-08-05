use sentc_crypto_common::user::{DoneLoginServerKeysOutput, RegisterData};
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

#[derive(Serialize, Deserialize)]
pub struct CustomerDoneRegistrationInput
{
	pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerDoneLoginOutput
{
	pub user_keys: DoneLoginServerKeysOutput,
	pub email_data: CustomerEmailData,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerEmailData
{
	pub validate_email: bool,
	pub email: String,
	pub email_send: u128,
	pub email_status: i32,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerUpdateInput
{
	pub new_email: String
}
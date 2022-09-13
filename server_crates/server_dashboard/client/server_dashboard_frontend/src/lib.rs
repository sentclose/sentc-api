#![no_std]

mod customer;

extern crate alloc;

use alloc::string::{String, ToString};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct CustomerEmailData
{
	validate_email: bool,
	email: String,
	email_send: u128,
	email_status: i32,
}

impl From<server_api_common::customer::CustomerEmailData> for CustomerEmailData
{
	fn from(data: server_api_common::customer::CustomerEmailData) -> Self
	{
		Self {
			validate_email: data.validate_email,
			email: data.email,
			email_send: data.email_send,
			email_status: data.email_status,
		}
	}
}

#[wasm_bindgen]
pub struct DoneLoginLightServerOutput
{
	user_id: String,
	jwt: String,
	device_id: String,
}

impl From<sentc_crypto_common::user::DoneLoginLightServerOutput> for DoneLoginLightServerOutput
{
	fn from(key: sentc_crypto_common::user::DoneLoginLightServerOutput) -> Self
	{
		Self {
			user_id: key.user_id,
			jwt: key.jwt,
			device_id: key.device_id,
		}
	}
}

#[wasm_bindgen]
pub struct CustomerDoneLoginOutput
{
	user_keys: DoneLoginLightServerOutput,
	email_data: CustomerEmailData,
}

impl From<server_api_common::customer::CustomerDoneLoginOutput> for CustomerDoneLoginOutput
{
	fn from(data: server_api_common::customer::CustomerDoneLoginOutput) -> Self
	{
		Self {
			user_keys: data.user_keys.into(),
			email_data: data.email_data.into(),
		}
	}
}

#[wasm_bindgen]
impl CustomerDoneLoginOutput
{
	pub fn get_email(&self) -> String
	{
		self.email_data.email.clone()
	}

	pub fn get_validate_email(&self) -> bool
	{
		self.email_data.validate_email
	}

	pub fn get_email_send(&self) -> String
	{
		self.email_data.email_send.to_string()
	}

	pub fn get_email_status(&self) -> i32
	{
		self.email_data.email_status
	}

	pub fn get_user_id(&self) -> String
	{
		self.user_keys.user_id.clone()
	}

	pub fn get_jwt(&self) -> String
	{
		self.user_keys.jwt.clone()
	}

	pub fn get_device_id(&self) -> String
	{
		self.user_keys.device_id.clone()
	}
}

#[wasm_bindgen]
pub async fn check_user_identifier_available(base_url: String, auth_token: String, user_identifier: String) -> Result<bool, JsValue>
{
	let out = sentc_crypto_full::user::check_user_identifier_available(base_url, auth_token.as_str(), user_identifier.as_str()).await?;

	Ok(out)
}

#[wasm_bindgen]
pub async fn register(base_url: String, auth_token: String, email: String, password: String) -> Result<String, JsValue>
{
	let out = customer::register(base_url, auth_token.as_str(), email, password.as_str()).await?;

	Ok(out)
}

#[wasm_bindgen]
pub async fn login(base_url: String, auth_token: String, email: String, password: String) -> Result<CustomerDoneLoginOutput, JsValue>
{
	let out = customer::login(base_url, auth_token.as_str(), email.as_str(), password.as_str()).await?;

	Ok(out.into())
}

#[wasm_bindgen]
pub async fn update(base_url: String, auth_token: String, jwt: String, new_email: String) -> Result<(), JsValue>
{
	Ok(customer::update(base_url, auth_token.as_str(), jwt.as_str(), new_email).await?)
}

#[wasm_bindgen]
pub async fn delete_customer(base_url: String, auth_token: String, email: String, pw: String) -> Result<(), JsValue>
{
	Ok(customer::delete_customer(base_url, auth_token.as_str(), email.as_str(), pw.as_str()).await?)
}

#[wasm_bindgen]
pub async fn prepare_reset_password(base_url: String, auth_token: String, email: String) -> Result<(), JsValue>
{
	Ok(customer::prepare_reset_password(base_url, auth_token.as_str(), email).await?)
}

#[wasm_bindgen]
pub async fn done_reset_password(base_url: String, auth_token: String, token: String, email: String, new_pw: String) -> Result<(), JsValue>
{
	Ok(customer::done_reset_password(base_url, auth_token.as_str(), token, email.as_str(), new_pw.as_str()).await?)
}

#[wasm_bindgen]
pub async fn change_password(base_url: String, auth_token: String, email: String, old_pw: String, new_pw: String) -> Result<(), JsValue>
{
	Ok(customer::change_password(
		base_url,
		auth_token.as_str(),
		email.as_str(),
		old_pw.as_str(),
		new_pw.as_str(),
	)
	.await?)
}

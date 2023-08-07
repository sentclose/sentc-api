pub mod app_controller;
pub mod app_entities;
pub(crate) mod app_model;
pub mod app_service;

use rand::RngCore;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};

use crate::ApiErrorCodes;

fn generate_tokens() -> Result<([u8; 50], [u8; 30]), ServerCoreError>
{
	let mut rng = rand::thread_rng();

	let mut secret_token = [0u8; 50];

	rng.try_fill_bytes(&mut secret_token)
		.map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::AppTokenWrongFormat, "Can't create secret token"))?;

	let mut public_token = [0u8; 30];

	rng.try_fill_bytes(&mut public_token)
		.map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::AppTokenWrongFormat, "Can't create secret token"))?;

	Ok((secret_token, public_token))
}

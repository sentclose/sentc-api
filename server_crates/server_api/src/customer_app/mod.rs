pub mod app_controller;
pub mod app_entities;
pub(crate) mod app_model;
pub mod app_service;
pub(crate) mod app_util;

use rand::RngCore;

pub(crate) use self::app_controller::*;
use crate::util::api_res::{ApiErrorCodes, HttpErr};

fn generate_tokens() -> Result<([u8; 50], [u8; 30]), HttpErr>
{
	let mut rng = rand::thread_rng();

	let mut secret_token = [0u8; 50];

	rng.try_fill_bytes(&mut secret_token).map_err(|_| {
		HttpErr::new(
			400,
			ApiErrorCodes::AppTokenWrongFormat,
			"Can't create secret token".to_string(),
			None,
		)
	})?;

	let mut public_token = [0u8; 30];

	rng.try_fill_bytes(&mut public_token).map_err(|_| {
		HttpErr::new(
			400,
			ApiErrorCodes::AppTokenWrongFormat,
			"Can't create secret token".to_string(),
			None,
		)
	})?;

	Ok((secret_token, public_token))
}

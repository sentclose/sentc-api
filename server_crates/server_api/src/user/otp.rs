use rand::RngCore;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::user::OtpRegister;
use totp_rs::{Algorithm, Secret, TOTP};

use crate::util::api_res::ApiErrorCodes;

pub const OTP_ALG: &str = "totp_sha256";

fn get_totp(sec: String) -> AppRes<TOTP>
{
	TOTP::new(
		Algorithm::SHA256,
		6,
		1,
		30,
		Secret::Encoded(sec)
			.to_bytes()
			.map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::ToTpSecretDecode, "Can't use the totp secret"))?,
	)
	.map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::ToTpGet, "Can't get the one time token"))
}

fn generate_totp_secret() -> AppRes<String>
{
	let mut rng = rand::thread_rng();
	let mut token = [0u8; 32];
	rng.try_fill_bytes(&mut token)
		.map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::AppTokenWrongFormat, "Can't create one time token"))?;

	let base32_string = base32::encode(
		base32::Alphabet::RFC4648 {
			padding: false,
		},
		&token,
	);

	//use totp to check the secret
	let totp = get_totp(base32_string)?;

	Ok(totp.get_secret_base32())
}

/**
Create the recovery keys for the 2fa if the user lost the device.

This keys should be printed out and can only be used once.

Do not use a hashed version because the user still need a possibility to print the keys again
*/
fn create_recover() -> AppRes<Vec<String>>
{
	let mut rng = rand::thread_rng();

	let mut vec = Vec::with_capacity(6);

	#[allow(clippy::needless_range_loop)]
	for _i in 0..6 {
		let mut recover = [0u8; 32];
		rng.try_fill_bytes(&mut recover)
			.map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::AppTokenWrongFormat, "Can't create one time token"))?;

		let base32_string = base32::encode(
			base32::Alphabet::RFC4648 {
				padding: false,
			},
			&recover,
		);

		vec.push(base32_string);
	}

	Ok(vec)
}

pub fn register_otp() -> AppRes<OtpRegister>
{
	let secret = generate_totp_secret()?;

	let recover = create_recover()?;

	Ok(OtpRegister {
		secret,
		alg: OTP_ALG.to_string(),
		recover,
	})
}

pub fn validate_otp(sec: String, token: &str) -> AppRes<bool>
{
	let totp = get_totp(sec)?;

	totp.check_current(token)
		.map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::ToTpSecretDecode, "No valid token"))
}

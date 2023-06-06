use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;

use crate::error::{EATErrorCodes, SentcSdkErrorWrapper};
use crate::{KeyData, CRYPTO_ROOT_KEY};

fn encrypt_key_err() -> ServerCoreError
{
	ServerCoreError::new_msg_and_debug(
		500,
		EATErrorCodes::KeyNotFound,
		"No key found for encryption",
		Some("There is no key to encrypt the data for encrypted at rest".to_string()),
	)
}

fn decrypt_key_error() -> ServerCoreError
{
	ServerCoreError::new_msg_and_debug(
		500,
		EATErrorCodes::KeyNotFound,
		"No key found for decryption",
		Some("There is no key to decrypt the data for encrypted at rest".to_string()),
	)
}

pub async fn encrypt(data: &str) -> AppRes<String>
{
	let key_data = CRYPTO_ROOT_KEY.get().unwrap().read().await;

	let key = key_data
		.map
		.get(key_data.newest_key_id.as_str())
		.ok_or_else(encrypt_key_err)?;

	sentc_crypto::crypto::encrypt_string_symmetric(key, data, None).map_err(|e| SentcSdkErrorWrapper(e).into())
}

pub async fn decrypt(encrypted: &str) -> AppRes<String>
{
	let key_data = CRYPTO_ROOT_KEY.get().unwrap().read().await;

	decrypt_with_key_map(&key_data, encrypted)
}

pub fn decrypt_with_key_map(key_data: &KeyData, encrypted: &str) -> AppRes<String>
{
	let head = sentc_crypto::crypto::split_head_and_encrypted_string(encrypted).map_err(|e| SentcSdkErrorWrapper(e).into())?;

	let key = key_data.map.get(&head.id).ok_or_else(decrypt_key_error)?;

	sentc_crypto::crypto::decrypt_string_symmetric(key, encrypted, None).map_err(|e| SentcSdkErrorWrapper(e).into())
}

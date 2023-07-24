use rustgram_server_util::res::AppRes;
use sentc_crypto::entities::keys::SymKeyFormatInt;

use crate::error::SentcSdkErrorWrapper;
use crate::CRYPTO_ROOT_KEY;

pub async fn encrypt(data: &str) -> AppRes<String>
{
	let key = CRYPTO_ROOT_KEY.get().unwrap().read().await;

	encrypt_with_key(&key, data)
}

pub fn encrypt_with_key(key: &SymKeyFormatInt, data: &str) -> AppRes<String>
{
	sentc_crypto::crypto::encrypt_string_symmetric(key, data, None).map_err(|e| SentcSdkErrorWrapper(e).into())
}

pub async fn decrypt(encrypted: &str) -> AppRes<String>
{
	let key = CRYPTO_ROOT_KEY.get().unwrap().read().await;

	decrypt_with_key(&key, encrypted)
}

pub fn decrypt_with_key(key: &SymKeyFormatInt, encrypted: &str) -> AppRes<String>
{
	sentc_crypto::crypto::decrypt_string_symmetric(key, encrypted, None).map_err(|e| SentcSdkErrorWrapper(e).into())
}

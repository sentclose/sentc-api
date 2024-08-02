use rustgram_server_util::res::AppRes;
use sentc_crypto::sdk_utils::cryptomat::SymKeyCrypto;
use sentc_crypto_std_keys::util::SymmetricKey;

use crate::error::SentcSdkErrorWrapper;
use crate::CRYPTO_ROOT_KEY;

pub async fn encrypt(data: &str) -> AppRes<String>
{
	let key = CRYPTO_ROOT_KEY.get().unwrap().read().await;

	encrypt_with_key(&key, data)
}

pub fn encrypt_with_key(key: &SymmetricKey, data: &str) -> AppRes<String>
{
	key.encrypt_string(data)
		.map_err(|e| SentcSdkErrorWrapper(e.into()).into())
}

pub async fn decrypt(encrypted: &str) -> AppRes<String>
{
	let key = CRYPTO_ROOT_KEY.get().unwrap().read().await;

	decrypt_with_key(&key, encrypted)
}

pub fn decrypt_with_key(key: &SymmetricKey, encrypted: &str) -> AppRes<String>
{
	key.decrypt_string(encrypted, None)
		.map_err(|e| SentcSdkErrorWrapper(e.into()).into())
}

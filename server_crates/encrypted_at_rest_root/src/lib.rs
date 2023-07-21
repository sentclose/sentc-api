mod encrypt;
pub mod error;
mod key_gen;

use std::env;

pub use encrypt::{decrypt, decrypt_with_key, encrypt, encrypt_with_key};
pub use key_gen::{export_key, generate_and_export_new_key, generate_new_key};
use sentc_crypto::sdk_utils::keys::{SymKeyFormatExport, SymKeyFormatInt};
use tokio::sync::{OnceCell, RwLock, RwLockReadGuard};

static CRYPTO_ROOT_KEY: OnceCell<RwLock<SymKeyFormatInt>> = OnceCell::const_new();

pub async fn init_crypto()
{
	CRYPTO_ROOT_KEY
		.get_or_init(init_private_crypto)
		.await
		.read()
		.await;
}

pub async fn get_key_map<'a>() -> RwLockReadGuard<'a, SymKeyFormatInt>
{
	CRYPTO_ROOT_KEY.get().unwrap().read().await
}

/**
Get the key from the root secret key.

The root key should be base64 encoded.

Use a fake key id.
*/
async fn init_private_crypto() -> RwLock<SymKeyFormatInt>
{
	let key = env::var("ROOT_KEY").unwrap();

	let key_export = SymKeyFormatExport::Aes {
		key,
		key_id: "n".to_string(),
	};

	let key = key_export.try_into().unwrap();

	RwLock::new(key)
}

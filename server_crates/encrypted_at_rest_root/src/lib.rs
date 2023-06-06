mod encrypt;
pub mod error;
mod key_gen;

use std::collections::HashMap;
use std::env;

pub use encrypt::{decrypt, decrypt_with_key_map, encrypt};
pub use key_gen::{delete_key, generate_and_add_new_key};
use sentc_crypto::entities::keys::{SymKeyFormatExport, SymKeyFormatInt};
use sentc_crypto::SdkError;
use tokio::sync::{OnceCell, RwLock, RwLockReadGuard};

static CRYPTO_ROOT_KEY: OnceCell<RwLock<KeyData>> = OnceCell::const_new();

pub struct KeyData
{
	newest_key_id: String,
	map: KeyMap,
}

type KeyMap = HashMap<String, SymKeyFormatInt>;

pub async fn init_crypto()
{
	CRYPTO_ROOT_KEY
		.get_or_init(init_private_crypto)
		.await
		.read()
		.await;
}

pub async fn get_key_map<'a>() -> RwLockReadGuard<'a, KeyData>
{
	CRYPTO_ROOT_KEY.get().unwrap().read().await
}

async fn init_private_crypto() -> RwLock<KeyData>
{
	let key = env::var("ROOT_KEYS").unwrap();

	let out: Vec<SymKeyFormatExport> = serde_json::from_str(&key).unwrap();

	let out: Vec<SymKeyFormatInt> = out
		.into_iter()
		.map(|k| k.try_into())
		.collect::<Result<_, SdkError>>()
		.unwrap();

	let newest_key_id = out[0].key_id.clone();

	let map: HashMap<String, SymKeyFormatInt> = out
		.into_iter()
		.map(|key| (key.key_id.clone(), key))
		.collect();

	RwLock::new(KeyData {
		newest_key_id,
		map,
	})
}

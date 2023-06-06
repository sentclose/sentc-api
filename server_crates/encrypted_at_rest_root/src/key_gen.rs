use sentc_crypto::entities::keys::{SymKeyFormatExport, SymKeyFormatInt};

fn new_key() -> SymKeyFormatInt
{
	let key = sentc_crypto::sdk_core::crypto::generate_symmetric().unwrap();

	let key_id = rustgram_server_util::db::id_handling::create_id();

	SymKeyFormatInt {
		key: key.key,
		key_id,
	}
}

/**
Cli app
 */
pub fn generate_and_add_new_key(old_keys: &str) -> String
{
	let new_key = new_key();
	let new_key: SymKeyFormatExport = new_key.into();

	let keys = if old_keys.is_empty() {
		//init new keys
		vec![new_key]
	} else {
		//get the old keys
		let mut old_keys: Vec<SymKeyFormatExport> = serde_json::from_str(old_keys).unwrap();

		old_keys.insert(0, new_key);

		old_keys
	};

	serde_json::to_string(&keys).unwrap()
}

pub fn delete_key(old_keys: &str, key_id: &str) -> String
{
	let mut old_keys: Vec<SymKeyFormatExport> = serde_json::from_str(old_keys).unwrap();

	let item = old_keys
		.iter()
		.map(|k| TryInto::<SymKeyFormatInt>::try_into(k).unwrap())
		.position(|i| i.key_id == key_id)
		.unwrap();

	old_keys.remove(item);

	serde_json::to_string(&old_keys).unwrap()
}

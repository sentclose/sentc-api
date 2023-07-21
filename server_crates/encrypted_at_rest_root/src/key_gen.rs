use sentc_crypto::sdk_utils::keys::{SymKeyFormatExport, SymKeyFormatInt};

pub fn generate_new_key() -> SymKeyFormatInt
{
	let key = sentc_crypto::sdk_core::crypto::generate_symmetric().unwrap();

	SymKeyFormatInt {
		key: key.key,
		key_id: "n".to_string(),
	}
}

pub fn export_key(key: SymKeyFormatInt) -> String
{
	let new_key: SymKeyFormatExport = key.into();

	match new_key {
		SymKeyFormatExport::Aes {
			key,
			key_id: _,
		} => key,
	}
}

/**
Cli app

Export only the base64 encoded key as string not the json string
 */
pub fn generate_and_export_new_key() -> String
{
	let new_key = generate_new_key();
	export_key(new_key)
}

use sentc_crypto::sdk_core::cryptomat::SymKeyGen;
use sentc_crypto_std_keys::util::{SymKeyFormatExport, SymmetricKey};

pub fn generate_new_key() -> SymmetricKey
{
	let key = sentc_crypto_std_keys::core::SymmetricKey::generate().unwrap();

	SymmetricKey {
		key,
		key_id: "n".to_string(),
	}
}

pub fn export_key(key: SymmetricKey) -> String
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

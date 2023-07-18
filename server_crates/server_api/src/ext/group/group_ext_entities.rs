use sentc_crypto_common::SymKeyId;

#[cfg_attr(feature = "mysql", derive(rustgram_server_util::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(rustgram_server_util::Sqlite))]
pub struct FetchedExt
{
	pub id: String,
	pub ext_name: String,
	pub ext_data: String,
	pub encrypted_key_id: SymKeyId,
	pub time: u128,
}

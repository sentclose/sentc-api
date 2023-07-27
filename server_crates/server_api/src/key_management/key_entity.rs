use rustgram_server_util::DB;
use sentc_crypto_common::crypto::GeneratedSymKeyHeadServerOutput;
use sentc_crypto_common::SymKeyId;
use serde::Serialize;

#[derive(Serialize, DB)]
pub struct SymKeyEntity
{
	key_id: SymKeyId,
	master_key_id: SymKeyId,
	encrypted_key_string: String,
	alg: String,
	time: u128,
}

impl Into<GeneratedSymKeyHeadServerOutput> for SymKeyEntity
{
	fn into(self) -> GeneratedSymKeyHeadServerOutput
	{
		GeneratedSymKeyHeadServerOutput {
			alg: self.alg,
			encrypted_key_string: self.encrypted_key_string,
			master_key_id: self.master_key_id,
			key_id: self.key_id,
			time: self.time,
		}
	}
}

//__________________________________________________________________________________________________

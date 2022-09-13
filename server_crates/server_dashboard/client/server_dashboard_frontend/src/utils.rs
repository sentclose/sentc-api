use alloc::string::String;

use sentc_crypto::SdkError;
use serde::Serialize;

pub fn to_string<T: ?Sized + Serialize>(obj: &T) -> Result<String, SdkError>
{
	serde_json::to_string(obj).map_err(|_e| SdkError::JsonToStringFailed)
}

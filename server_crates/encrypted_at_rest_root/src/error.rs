use rustgram_server_util::error::{ServerCoreError, ServerErrorCodes, ServerErrorConstructor};

pub enum EATErrorCodes
{
	KeyNotFound,
	SdkError,
}

impl ServerErrorCodes for EATErrorCodes
{
	fn get_int_code(&self) -> u32
	{
		match self {
			EATErrorCodes::KeyNotFound => 10000,
			EATErrorCodes::SdkError => 10001,
		}
	}
}

pub struct SentcSdkErrorWrapper(pub sentc_crypto::SdkError);

impl From<sentc_crypto::SdkError> for SentcSdkErrorWrapper
{
	fn from(value: sentc_crypto::SdkError) -> Self
	{
		Self(value)
	}
}

#[allow(clippy::from_over_into)]
impl Into<ServerCoreError> for SentcSdkErrorWrapper
{
	fn into(self) -> ServerCoreError
	{
		let msg = sentc_crypto::err_to_msg(self.0);

		ServerCoreError::new_msg_owned(400, EATErrorCodes::SdkError, msg, None)
	}
}

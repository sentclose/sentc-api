use rustgram_server_util::{take_or_err, DB};
use sentc_crypto_common::user::{UserInitServerOutput, UserPublicKeyDataServerOutput, UserVerifyKeyDataServerOutput};
use sentc_crypto_common::{DeviceId, EncryptionKeyPairId, GroupId, SignKeyPairId, UserId};
use serde::{Deserialize, Serialize};

use crate::group::group_entities::{GroupInviteReq, GroupUserKeys};
use crate::sentc_group_entities::GroupHmacData;

//generated with browser console: btoa(String.fromCharCode.apply(null, window.crypto.getRandomValues(new Uint8Array(128/8))));
//the value with the used alg
pub const SERVER_RANDOM_VALUE: (&str, &str) = ("zx4AKPCMHkeZnh21ciQ62w==", sentc_crypto::util::public::ARGON_2_OUTPUT);

//__________________________________________________________________________________________________
//User login data

#[derive(Serialize, Deserialize, DB)]
pub struct UserLoginDataEntity
{
	pub client_random_value: String,
	pub hashed_authentication_key: String,
	pub derived_alg: String,
}

#[derive(Serialize, Deserialize, DB)]
pub struct UserLoginDataOtpEntity
{
	pub client_random_value: String,
	pub hashed_authentication_key: String,
	pub derived_alg: String,

	//this shows if the user enabled the otp for 2fa
	pub otp_secret: Option<String>,
	pub otp_alg: Option<String>,
}

//__________________________________________________________________________________________________
//User done login data

#[derive(Serialize)]
pub struct LoginForcedOutput
{
	pub device_keys: VerifyLoginForcedEntity,
	pub verify: VerifyLoginOutput,
}

impl Into<sentc_crypto_common::user::LoginForcedOutput> for LoginForcedOutput
{
	fn into(self) -> sentc_crypto_common::user::LoginForcedOutput
	{
		sentc_crypto_common::user::LoginForcedOutput {
			device_keys: self.device_keys.into(),
			verify: self.verify.into(),
		}
	}
}

#[derive(Serialize)]
pub struct LoginForcedLightOutput
{
	pub device_keys: VerifyLoginForcedEntity,
	pub verify: sentc_crypto_common::user::VerifyLoginLightOutput,
}

impl Into<sentc_crypto_common::user::LoginForcedLightOutput> for LoginForcedLightOutput
{
	fn into(self) -> sentc_crypto_common::user::LoginForcedLightOutput
	{
		sentc_crypto_common::user::LoginForcedLightOutput {
			device_keys: self.device_keys.into(),
			verify: self.verify,
		}
	}
}

#[derive(Serialize)]
pub struct DoneLoginServerOutput
{
	pub device_keys: DoneLoginServerKeysOutputEntity,
	pub challenge: String,
}

impl Into<sentc_crypto_common::user::DoneLoginServerOutput> for DoneLoginServerOutput
{
	fn into(self) -> sentc_crypto_common::user::DoneLoginServerOutput
	{
		sentc_crypto_common::user::DoneLoginServerOutput {
			device_keys: self.device_keys.into(),
			challenge: self.challenge,
		}
	}
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize)]
pub enum DoneLoginServerReturn
{
	Otp,
	Direct(DoneLoginServerOutput),
}

impl Into<sentc_crypto_common::user::DoneLoginServerReturn> for DoneLoginServerReturn
{
	fn into(self) -> sentc_crypto_common::user::DoneLoginServerReturn
	{
		match self {
			Self::Otp => sentc_crypto_common::user::DoneLoginServerReturn::Otp,
			Self::Direct(d) => sentc_crypto_common::user::DoneLoginServerReturn::Direct(d.into()),
		}
	}
}

#[derive(Serialize)]
pub struct VerifyLoginOutput
{
	pub user_keys: Vec<GroupUserKeys>,
	pub hmac_keys: Vec<GroupHmacData>,
	pub jwt: String,
	pub refresh_token: String,
}

impl Into<sentc_crypto_common::user::VerifyLoginOutput> for VerifyLoginOutput
{
	fn into(self) -> sentc_crypto_common::user::VerifyLoginOutput
	{
		sentc_crypto_common::user::VerifyLoginOutput {
			jwt: self.jwt,
			refresh_token: self.refresh_token,
			user_keys: self.user_keys.into_iter().map(|k| k.into()).collect(),
			hmac_keys: self.hmac_keys.into_iter().map(|k| k.into()).collect(),
		}
	}
}

#[derive(Serialize)]
pub struct DoneLoginServerKeysOutputEntity
{
	pub encrypted_master_key: String,
	pub encrypted_private_key: String,
	pub public_key_string: String,
	pub keypair_encrypt_alg: String,
	pub encrypted_sign_key: String,
	pub verify_key_string: String,
	pub keypair_sign_alg: String,
	pub keypair_encrypt_id: EncryptionKeyPairId,
	pub keypair_sign_id: SignKeyPairId,
	pub user_id: UserId,
	pub device_id: DeviceId,
	pub user_group_id: GroupId,
}

impl Into<sentc_crypto_common::user::DoneLoginServerKeysOutput> for DoneLoginServerKeysOutputEntity
{
	fn into(self) -> sentc_crypto_common::user::DoneLoginServerKeysOutput
	{
		sentc_crypto_common::user::DoneLoginServerKeysOutput {
			encrypted_master_key: self.encrypted_master_key,
			encrypted_private_key: self.encrypted_private_key,
			public_key_string: self.public_key_string,
			keypair_encrypt_alg: self.keypair_encrypt_alg,
			encrypted_sign_key: self.encrypted_sign_key,
			verify_key_string: self.verify_key_string,
			keypair_sign_alg: self.keypair_sign_alg,
			keypair_encrypt_id: self.keypair_encrypt_id,
			keypair_sign_id: self.keypair_sign_id,
			user_id: self.user_id,
			device_id: self.device_id,
			user_group_id: self.user_group_id,
		}
	}
}

#[cfg(feature = "mysql")]
impl rustgram_server_util::db::mysql_async_export::prelude::FromRow for DoneLoginServerKeysOutputEntity
{
	fn from_row_opt(
		mut row: rustgram_server_util::db::mysql_async_export::Row,
	) -> Result<Self, rustgram_server_util::db::mysql_async_export::FromRowError>
	where
		Self: Sized,
	{
		let k_id: String = take_or_err!(row, 7, String);
		let keypair_encrypt_id = k_id.to_string();
		let keypair_sign_id = k_id.to_string();

		Ok(Self {
			encrypted_master_key: take_or_err!(row, 0, String),
			encrypted_private_key: take_or_err!(row, 1, String),
			public_key_string: take_or_err!(row, 2, String),
			keypair_encrypt_alg: take_or_err!(row, 3, String),
			encrypted_sign_key: take_or_err!(row, 4, String),
			verify_key_string: take_or_err!(row, 5, String),
			keypair_sign_alg: take_or_err!(row, 6, String),
			keypair_encrypt_id,
			keypair_sign_id,
			user_id: take_or_err!(row, 8, String),
			device_id: k_id,
			user_group_id: take_or_err!(row, 9, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl rustgram_server_util::db::FromSqliteRow for DoneLoginServerKeysOutputEntity
{
	fn from_row_opt(row: &rustgram_server_util::db::rusqlite_export::Row) -> Result<Self, rustgram_server_util::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let k_id: String = take_or_err!(row, 7);
		let keypair_encrypt_id = k_id.to_string();
		let keypair_sign_id = k_id.to_string();

		Ok(Self {
			encrypted_master_key: take_or_err!(row, 0),
			encrypted_private_key: take_or_err!(row, 1),
			public_key_string: take_or_err!(row, 2),
			keypair_encrypt_alg: take_or_err!(row, 3),
			encrypted_sign_key: take_or_err!(row, 4),
			verify_key_string: take_or_err!(row, 5),
			keypair_sign_alg: take_or_err!(row, 6),
			keypair_encrypt_id,
			keypair_sign_id,
			user_id: take_or_err!(row, 8),
			device_id: k_id,
			user_group_id: take_or_err!(row, 9),
		})
	}
}

#[derive(DB)]
pub struct VerifyLoginEntity
{
	pub device_id: DeviceId,
	pub user_id: UserId,
	pub user_group_id: GroupId,
}

#[derive(DB, Serialize)]
pub struct VerifyLoginForcedEntity
{
	pub device_id: DeviceId,
	pub user_id: UserId,
	pub user_group_id: GroupId,

	pub encrypted_master_key: String,
	pub encrypted_private_key: String,
	pub public_key_string: String,
	pub keypair_encrypt_alg: String,
	pub encrypted_sign_key: String,
	pub verify_key_string: String,
	pub keypair_sign_alg: String,
}

impl Into<sentc_crypto_common::user::VerifyLoginForcedEntity> for VerifyLoginForcedEntity
{
	fn into(self) -> sentc_crypto_common::user::VerifyLoginForcedEntity
	{
		sentc_crypto_common::user::VerifyLoginForcedEntity {
			device_id: self.device_id,
			user_id: self.user_id,
			user_group_id: self.user_group_id,
			encrypted_master_key: self.encrypted_master_key,
			encrypted_private_key: self.encrypted_private_key,
			public_key_string: self.public_key_string,
			keypair_encrypt_alg: self.keypair_encrypt_alg,
			encrypted_sign_key: self.encrypted_sign_key,
			verify_key_string: self.verify_key_string,
			keypair_sign_alg: self.keypair_sign_alg,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(DB)]
pub struct UserRefreshTokenCheck
{
	pub user_id: DeviceId,
	pub device_identifier: String,
}

//__________________________________________________________________________________________________

#[derive(Serialize, DB)]
pub struct UserPublicKeyDataEntity
{
	pub public_key_id: EncryptionKeyPairId,
	pub public_key: String,
	pub public_key_alg: String,
	pub public_key_sig: Option<String>,
	pub public_key_sig_key_id: Option<String>,
}

impl Into<UserPublicKeyDataServerOutput> for UserPublicKeyDataEntity
{
	fn into(self) -> UserPublicKeyDataServerOutput
	{
		UserPublicKeyDataServerOutput {
			public_key_id: self.public_key_id,
			public_key: self.public_key,
			public_key_alg: self.public_key_alg,
			public_key_sig: self.public_key_sig,
			public_key_sig_key_id: self.public_key_sig_key_id,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, DB)]
pub struct UserVerifyKeyDataEntity
{
	pub verify_key_id: EncryptionKeyPairId,
	pub verify_key: String,
	pub verify_key_alg: String,
}

impl Into<UserVerifyKeyDataServerOutput> for UserVerifyKeyDataEntity
{
	fn into(self) -> UserVerifyKeyDataServerOutput
	{
		UserVerifyKeyDataServerOutput {
			verify_key_id: self.verify_key_id,
			verify_key: self.verify_key,
			verify_key_alg: self.verify_key_alg,
		}
	}
}

//__________________________________________________________________________________________________

/**
Only used in the controller
*/
#[derive(Serialize)]
pub struct UserInitEntity
{
	pub jwt: String,
	pub invites: Vec<GroupInviteReq>,
}

impl Into<UserInitServerOutput> for UserInitEntity
{
	fn into(self) -> UserInitServerOutput
	{
		let mut invites = Vec::with_capacity(self.invites.len());

		for invite in self.invites {
			invites.push(invite.into());
		}

		UserInitServerOutput {
			jwt: self.jwt,
			invites,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, DB)]
pub struct UserDeviceList
{
	pub device_id: String,
	pub time: u128,
	pub device_identifier: String,
}

impl Into<sentc_crypto_common::user::UserDeviceList> for UserDeviceList
{
	fn into(self) -> sentc_crypto_common::user::UserDeviceList
	{
		sentc_crypto_common::user::UserDeviceList {
			device_id: self.device_id,
			time: self.time,
			device_identifier: self.device_identifier,
		}
	}
}

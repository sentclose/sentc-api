use sentc_crypto_common::user::{UserInitServerOutput, UserPublicKeyDataServerOutput, UserVerifyKeyDataServerOutput};
use sentc_crypto_common::{DeviceId, EncryptionKeyPairId, GroupId, SignKeyPairId, UserId};
use serde::{Deserialize, Serialize};
use server_core::take_or_err;

use crate::group::group_entities::{GroupInviteReq, GroupUserKeys};

//generated with browser console: btoa(String.fromCharCode.apply(null, window.crypto.getRandomValues(new Uint8Array(128/8))));
//the value with the used alg
pub static SERVER_RANDOM_VALUE: (&'static str, &'static str) = ("zx4AKPCMHkeZnh21ciQ62w==", sentc_crypto::util::public::ARGON_2_OUTPUT);

//__________________________________________________________________________________________________
//Jwt

pub struct JwtSignKey(pub String);

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for JwtSignKey
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(JwtSignKey(take_or_err!(row, 0, String)))
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for JwtSignKey
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(JwtSignKey(take_or_err!(row, 0)))
	}
}

pub struct JwtVerifyKey(pub String);

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for JwtVerifyKey
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(JwtVerifyKey(take_or_err!(row, 0, String)))
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for JwtVerifyKey
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(JwtVerifyKey(take_or_err!(row, 0)))
	}
}

#[derive(Serialize, Deserialize)]
pub struct UserJwtEntity
{
	pub id: UserId,
	pub device_id: DeviceId,
	pub group_id: GroupId,
	pub identifier: String,
	//aud if it is an app user or an customer
	pub aud: String,
	pub sub: String, //the app id
	pub fresh: bool,
}

//__________________________________________________________________________________________________
//User exists

#[derive(Serialize, Deserialize)]
pub struct UserExistsEntity(pub i64); //i64 for sqlite

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserExistsEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(UserExistsEntity(take_or_err!(row, 0, i64)))
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for UserExistsEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(UserExistsEntity(take_or_err!(row, 0)))
	}
}

//__________________________________________________________________________________________________
//User login data

#[derive(Serialize, Deserialize)]
pub struct UserLoginDataEntity
{
	pub client_random_value: String,
	pub hashed_authentication_key: String,
	pub derived_alg: String,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserLoginDataEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(UserLoginDataEntity {
			client_random_value: take_or_err!(row, 0, String),
			hashed_authentication_key: take_or_err!(row, 1, String),
			derived_alg: take_or_err!(row, 2, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for UserLoginDataEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(UserLoginDataEntity {
			client_random_value: take_or_err!(row, 0),
			hashed_authentication_key: take_or_err!(row, 1),
			derived_alg: take_or_err!(row, 2),
		})
	}
}

//__________________________________________________________________________________________________
//User done login data

#[derive(Serialize)]
pub struct DoneLoginServerOutput
{
	pub device_keys: DoneLoginServerKeysOutputEntity,
	pub user_keys: Vec<GroupUserKeys>,
	pub jwt: String,
	pub refresh_token: String,
}

impl Into<sentc_crypto_common::user::DoneLoginServerOutput> for DoneLoginServerOutput
{
	fn into(self) -> sentc_crypto_common::user::DoneLoginServerOutput
	{
		let mut user_keys = Vec::with_capacity(self.user_keys.len());

		for user_key in self.user_keys {
			user_keys.push(user_key.into());
		}

		sentc_crypto_common::user::DoneLoginServerOutput {
			device_keys: self.device_keys.into(),
			jwt: self.jwt,
			refresh_token: self.refresh_token,
			user_keys,
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
impl mysql_async::prelude::FromRow for DoneLoginServerKeysOutputEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
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
			device_id: k_id.to_string(),
			user_group_id: take_or_err!(row, 9, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for DoneLoginServerKeysOutputEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
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
			device_id: k_id.to_string(),
			user_group_id: take_or_err!(row, 9),
		})
	}
}

//__________________________________________________________________________________________________

pub struct UserLoginLightEntity
{
	pub user_id: UserId,
	pub device_id: DeviceId,
	pub group_id: GroupId,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserLoginLightEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			user_id: take_or_err!(row, 0, String),
			device_id: take_or_err!(row, 1, String),
			group_id: take_or_err!(row, 2, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for UserLoginLightEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			user_id: take_or_err!(row, 0),
			device_id: take_or_err!(row, 1),
			group_id: take_or_err!(row, 2),
		})
	}
}

//__________________________________________________________________________________________________

pub struct UserResetPwCheck(pub UserId);

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserResetPwCheck
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0, String)))
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for UserResetPwCheck
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(take_or_err!(row, 0)))
	}
}

//__________________________________________________________________________________________________

pub struct UserRefreshTokenCheck
{
	pub user_id: DeviceId,
	pub device_identifier: String,
	pub group_id: GroupId,
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserRefreshTokenCheck
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			user_id: take_or_err!(row, 0, String),
			device_identifier: take_or_err!(row, 1, String),
			group_id: take_or_err!(row, 2, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for UserRefreshTokenCheck
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			user_id: take_or_err!(row, 0),
			device_identifier: take_or_err!(row, 1),
			group_id: take_or_err!(row, 2),
		})
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize)]
pub struct UserPublicData
{
	pub public_key_id: EncryptionKeyPairId,
	pub public_key: String,
	pub public_key_alg: String,

	pub verify_key_id: SignKeyPairId,
	pub verify_key: String,
	pub verify_alg: String,
}

impl Into<sentc_crypto_common::user::UserPublicData> for UserPublicData
{
	fn into(self) -> sentc_crypto_common::user::UserPublicData
	{
		sentc_crypto_common::user::UserPublicData {
			public_key_id: self.public_key_id,
			public_key: self.public_key,
			public_key_alg: self.public_key_alg,
			verify_key_id: self.verify_key_id,
			verify_key: self.verify_key,
			verify_alg: self.verify_alg,
		}
	}
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserPublicData
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		let k_id = take_or_err!(row, 0, String);

		Ok(Self {
			public_key_id: k_id.to_string(),
			public_key: take_or_err!(row, 1, String),
			public_key_alg: take_or_err!(row, 2, String),

			verify_key_id: k_id.to_string(),
			verify_key: take_or_err!(row, 3, String),
			verify_alg: take_or_err!(row, 4, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for UserPublicData
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let k_id: String = take_or_err!(row, 0);

		Ok(Self {
			public_key_id: k_id.to_string(),
			public_key: take_or_err!(row, 1),
			public_key_alg: take_or_err!(row, 2),

			verify_key_id: k_id.to_string(),
			verify_key: take_or_err!(row, 3),
			verify_alg: take_or_err!(row, 4),
		})
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize)]
pub struct UserPublicKeyDataEntity
{
	pub public_key_id: EncryptionKeyPairId,
	pub public_key: String,
	pub public_key_alg: String,
}

impl Into<UserPublicKeyDataServerOutput> for UserPublicKeyDataEntity
{
	fn into(self) -> UserPublicKeyDataServerOutput
	{
		UserPublicKeyDataServerOutput {
			public_key_id: self.public_key_id,
			public_key: self.public_key,
			public_key_alg: self.public_key_alg,
		}
	}
}

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserPublicKeyDataEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			public_key_id: take_or_err!(row, 0, String),
			public_key: take_or_err!(row, 1, String),
			public_key_alg: take_or_err!(row, 2, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for UserPublicKeyDataEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			public_key_id: take_or_err!(row, 0),
			public_key: take_or_err!(row, 1),
			public_key_alg: take_or_err!(row, 2),
		})
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize)]
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

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserVerifyKeyDataEntity
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			verify_key_id: take_or_err!(row, 0, String),
			verify_key: take_or_err!(row, 1, String),
			verify_key_alg: take_or_err!(row, 2, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for UserVerifyKeyDataEntity
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			verify_key_id: take_or_err!(row, 0),
			verify_key: take_or_err!(row, 1),
			verify_key_alg: take_or_err!(row, 2),
		})
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

#[derive(Serialize)]
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

#[cfg(feature = "mysql")]
impl mysql_async::prelude::FromRow for UserDeviceList
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			device_id: take_or_err!(row, 0, String),
			time: take_or_err!(row, 1, u128),
			device_identifier: take_or_err!(row, 2, String),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for UserDeviceList
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			device_id: take_or_err!(row, 0),
			time: server_core::take_or_err_u128!(row, 1),
			device_identifier: take_or_err!(row, 2),
		})
	}
}

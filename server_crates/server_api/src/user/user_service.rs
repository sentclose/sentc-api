use std::future::Future;

use rand::RngCore;
use rustgram_server_util::cache;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::user::{
	ChangePasswordData,
	DoneLoginLightServerOutput,
	JwtRefreshInput,
	KeyDerivedData,
	OtpRecoveryKeysOutput,
	OtpRegister,
	RegisterData,
	RegisterServerOutput,
	UserDeviceDoneRegisterInput,
	UserDeviceRegisterInput,
	UserDeviceRegisterOutput,
	UserIdentifierAvailableServerInput,
	UserIdentifierAvailableServerOutput,
	UserUpdateServerInput,
	VerifyLoginInput,
};
use sentc_crypto_common::{AppId, DeviceId, EncryptionKeyPairId, GroupId, SignKeyPairId, SymKeyId, UserId};
use server_api_common::customer_app::app_entities::AppData;
use server_api_common::group::group_entities::{InternalGroupData, InternalGroupDataComplete, InternalUserGroupData};
use server_api_common::group::GROUP_TYPE_USER;
use server_api_common::user::jwt::create_jwt;
use server_api_common::user::user_entity::UserJwtEntity;
use server_api_common::util::{get_user_in_app_key, hash_token_to_string};
use server_key_store::KeyStorage;

pub use self::user_model::{get_devices, get_group_key_rotations_in_actual_month, get_user_group_id, reset_password, save_user_action};
use crate::group::group_entities::GroupUserKeys;
use crate::group::group_user_service::NewUserType;
use crate::group::{group_service, group_user_service};
use crate::sentc_user_entities::{LoginForcedOutput, UserPublicKeyDataEntity, UserVerifyKeyDataEntity, VerifyLoginOutput};
use crate::user::auth::auth_service::{auth_user, verify_login_forced_internally, verify_login_internally};
use crate::user::user_entities::UserInitEntity;
use crate::user::user_model::DeviceForDelete;
use crate::user::{otp, user_model};
use crate::util::api_res::ApiErrorCodes;

#[macro_export]
macro_rules! check_user_group_keys_set {
	($encrypted_sign_key:expr, $verify_key:expr, $public_key_sig:expr, $keypair_sign_alg:expr) => {
		if $public_key_sig.is_none() || $verify_key.is_none() || $encrypted_sign_key.is_none() || $keypair_sign_alg.is_none() {
			return Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::UserKeysNotFound,
				"User keys not found. Make sure to create the user group.",
			));
		}
	};
}

pub enum UserAction
{
	Login,
	Refresh,
	Init,
	ChangePassword,
	ResetPassword,
	Delete,
	KeyRotation,
}

impl UserAction
{
	pub fn get_int_code(&self) -> u32
	{
		match self {
			UserAction::Login => 0,
			UserAction::Refresh => 1,
			UserAction::ChangePassword => 2,
			UserAction::ResetPassword => 3,
			UserAction::Delete => 4,
			UserAction::Init => 5,
			UserAction::KeyRotation => 6,
		}
	}
}

pub async fn exists(app_id: impl Into<AppId>, data: UserIdentifierAvailableServerInput) -> AppRes<UserIdentifierAvailableServerOutput>
{
	let identifier = hash_token_to_string(data.user_identifier.as_bytes())?;

	let exists = user_model::check_user_exists(app_id, &identifier).await?;

	let out = UserIdentifierAvailableServerOutput {
		user_identifier: data.user_identifier,
		available: !exists,
	};

	Ok(out)
}

pub async fn register(app_id: impl Into<AppId>, register_input: RegisterData) -> AppRes<RegisterServerOutput>
{
	let mut group_data = register_input.group;

	check_user_group_keys_set!(
		group_data.encrypted_sign_key,
		group_data.verify_key,
		group_data.public_key_sig,
		group_data.keypair_sign_alg
	);

	let device_data = register_input.device;

	let app_id = app_id.into();

	//save the data

	let identifier = hash_token_to_string(device_data.device_identifier.as_bytes())?;

	//mark the post quantum keys as external. the key fetch knows then to fetch the keys from the key store
	//no need for the normal keys like aes
	let derived = KeyDerivedData {
		derived_alg: device_data.derived.derived_alg,
		client_random_value: device_data.derived.client_random_value,
		hashed_authentication_key: device_data.derived.hashed_authentication_key,
		public_key: "extern".to_string(),
		encrypted_private_key: "extern".to_string(),
		keypair_encrypt_alg: device_data.derived.keypair_encrypt_alg,
		verify_key: "extern".to_string(),
		encrypted_sign_key: "extern".to_string(),
		keypair_sign_alg: device_data.derived.keypair_sign_alg,
	};

	let (user_id, device_id) = user_model::register(&app_id, identifier, device_data.master_key, derived).await?;

	server_key_store::upload_key(vec![
		KeyStorage {
			key: device_data.derived.public_key,
			id: format!("pk_{device_id}"),
		},
		KeyStorage {
			key: device_data.derived.encrypted_private_key,
			id: format!("sk_{device_id}"),
		},
		KeyStorage {
			key: device_data.derived.verify_key,
			id: format!("vk_{device_id}"),
		},
		KeyStorage {
			key: device_data.derived.encrypted_sign_key,
			id: format!("sign_k_{device_id}"),
		},
	])
	.await?;

	//update creator public key id in group data (with the device id), this is needed to know what public key was used to encrypt the group key
	group_data.creator_public_key_id = device_id.to_string();

	//create user group, insert the device not the user id because the devices are in the group not the user!
	let group_id = group_service::create_group(
		&app_id,
		&device_id,
		group_data,
		GROUP_TYPE_USER,
		None,
		None,
		None,
		false,
	)
	.await?
	.0;

	//delete the user in app check cache from the jwt mw
	//it can happen that a user id was used before which doesn't exist yet
	let cache_key = get_user_in_app_key(&app_id, &user_id);
	cache::delete(&cache_key).await?;

	//now update the user group id
	user_model::register_update_user_group_id(app_id, &user_id, group_id).await?;

	let out = RegisterServerOutput {
		user_id,
		device_id,
		device_identifier: device_data.device_identifier,
	};

	Ok(out)
}

/**
# Prepare the device

1. save the device keys
2. return the device id

In the client:
- transport the token to the active device
- call done register device with the device id and the token
*/
pub async fn prepare_register_device(app_id: impl Into<AppId>, input: UserDeviceRegisterInput) -> AppRes<UserDeviceRegisterOutput>
{
	let app_id = app_id.into();

	let identifier = hash_token_to_string(input.device_identifier.as_bytes())?;

	let check = user_model::check_user_exists(&app_id, &identifier).await?;

	if check {
		//check true == user exists
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::UserExists,
			"Identifier already exists",
		));
	}

	let public_key_string = input.derived.public_key.to_string();
	let keypair_encrypt_alg = input.derived.keypair_encrypt_alg.to_string();

	let token = create_refresh_token()?;

	let derived = KeyDerivedData {
		derived_alg: input.derived.derived_alg,
		client_random_value: input.derived.client_random_value,
		hashed_authentication_key: input.derived.hashed_authentication_key,
		public_key: "extern".to_string(),
		encrypted_private_key: "extern".to_string(),
		keypair_encrypt_alg: input.derived.keypair_encrypt_alg,
		verify_key: "extern".to_string(),
		encrypted_sign_key: "extern".to_string(),
		keypair_sign_alg: input.derived.keypair_sign_alg,
	};

	let device_id = user_model::register_device(app_id, identifier, input.master_key, derived, &token).await?;

	server_key_store::upload_key(vec![
		KeyStorage {
			key: input.derived.public_key,
			id: format!("pk_{device_id}"),
		},
		KeyStorage {
			key: input.derived.encrypted_private_key,
			id: format!("sk_{device_id}"),
		},
		KeyStorage {
			key: input.derived.verify_key,
			id: format!("vk_{device_id}"),
		},
		KeyStorage {
			key: input.derived.encrypted_sign_key,
			id: format!("sign_k_{device_id}"),
		},
	])
	.await?;

	Ok(UserDeviceRegisterOutput {
		device_id,
		token,
		device_identifier: input.device_identifier,
		public_key_string,
		keypair_encrypt_alg,
	})
}

/**
# Done the register device

In the client:
- prepare the user group keys

1. auto invite the new device
2. same as group auto invite
*/
pub async fn done_register_device(
	app_id: impl Into<AppId>,
	user_id: impl Into<UserId>,
	user_group_id: impl Into<GroupId>,
	input: UserDeviceDoneRegisterInput,
) -> AppRes<Option<String>>
{
	let app_id = app_id.into();

	let device_id = user_model::get_done_register_device(&app_id, input.token).await?;

	//for the auto invite we only need the group id and the group user rank
	let session_id = group_user_service::invite_auto(
		&internal_group_data(&app_id, user_group_id, 0),
		input.user_keys,
		&device_id, //invite the new device
		NewUserType::Normal,
		false,
	)
	.await?;

	user_model::done_register_device(app_id, user_id, device_id).await?;

	Ok(session_id)
}

//__________________________________________________________________________________________________

pub async fn verify_login(app_data: &AppData, done_login: VerifyLoginInput) -> AppRes<VerifyLoginOutput>
{
	let (data, jwt, refresh_token) = verify_login_internally(app_data, done_login).await?;

	//fetch the first page of the group keys with the device id as user
	let user_keys = group_service::get_user_group_keys(&app_data.app_data.app_id, &data.user_group_id, &data.device_id, 0, "").await?;

	let hmac_keys = group_service::get_group_hmac(
		&app_data.app_data.app_id,
		&data.user_group_id,
		0, //fetch the first page
		"",
	)
	.await?;

	//fetch the first page of the hmac keys

	let out = VerifyLoginOutput {
		user_keys,
		hmac_keys,
		jwt,
		refresh_token,
	};

	Ok(out)
}

pub async fn verify_login_forced(app_data: &AppData, identifier: &str) -> AppRes<LoginForcedOutput>
{
	let (data, jwt, refresh_token) = verify_login_forced_internally(app_data, identifier).await?;

	//fetch the first page of the group keys with the device id as user
	let user_keys = group_service::get_user_group_keys(&app_data.app_data.app_id, &data.user_group_id, &data.device_id, 0, "").await?;

	let hmac_keys = group_service::get_group_hmac(
		&app_data.app_data.app_id,
		&data.user_group_id,
		0, //fetch the first page
		"",
	)
	.await?;

	//fetch the first page of the hmac keys

	let out = VerifyLoginOutput {
		user_keys,
		hmac_keys,
		jwt,
		refresh_token,
	};

	Ok(LoginForcedOutput {
		device_keys: data,
		verify: out,
	})
}

pub fn get_user_keys<'a>(
	user: &'a UserJwtEntity,
	app_id: impl Into<AppId> + 'a,
	last_fetched_time: u128,
	last_k_id: impl Into<SymKeyId> + 'a,
) -> impl Future<Output = AppRes<Vec<GroupUserKeys>>> + 'a
{
	group_service::get_user_group_keys(
		app_id,
		&user.group_id,
		&user.device_id, //call it with the device id to decrypt the keys
		last_fetched_time,
		last_k_id,
	)
}

pub fn get_user_key<'a>(
	user: &'a UserJwtEntity,
	app_id: impl Into<AppId> + 'a,
	key_id: impl Into<SymKeyId> + 'a,
) -> impl Future<Output = AppRes<GroupUserKeys>> + 'a
{
	group_service::get_user_group_key(
		app_id,
		&user.group_id,
		&user.device_id, //call it with the device id to decrypt the keys
		key_id,
	)
}

pub(crate) async fn get_public_key_extern(out: &mut UserPublicKeyDataEntity) -> AppRes<()>
{
	let mut keys_to_fetch = vec![];

	if out.public_key == "extern" {
		keys_to_fetch.push(format!("pk_{}", out.public_key_id));
	}

	if let Some(sig) = out.public_key_sig.as_ref() {
		if sig == "extern" {
			keys_to_fetch.push(format!("sig_pk_{}", out.public_key_id));
		}
	}

	if keys_to_fetch.is_empty() {
		return Ok(());
	}

	let mut fetched_key = server_key_store::get_keys(&keys_to_fetch).await?;

	if out.public_key == "extern" {
		if let Some(fetched_key) = fetched_key.remove(&format!("pk_{}", out.public_key_id)) {
			out.public_key = fetched_key;
		}
	}

	if let Some(sig) = out.public_key_sig.as_ref() {
		if sig == "extern" {
			if let Some(fetched_key) = fetched_key.remove(&format!("sig_pk_{}", out.public_key_id)) {
				out.public_key_sig = Some(fetched_key);
			}
		}
	}

	Ok(())
}

pub async fn get_public_key_by_id(
	app_id: impl Into<AppId>,
	user_id: impl Into<UserId>,
	public_key_id: impl Into<EncryptionKeyPairId>,
) -> AppRes<UserPublicKeyDataEntity>
{
	let mut out = user_model::get_public_key_by_id(app_id, user_id, public_key_id).await?;

	get_public_key_extern(&mut out).await?;

	Ok(out)
}

pub async fn get_public_key_data(app_id: impl Into<AppId>, user_id: impl Into<UserId>) -> AppRes<UserPublicKeyDataEntity>
{
	let mut out = user_model::get_public_key_data(app_id, user_id).await?;

	get_public_key_extern(&mut out).await?;

	Ok(out)
}

pub async fn get_verify_key_by_id(
	app_id: impl Into<AppId>,
	user_id: impl Into<UserId>,
	verify_key_id: impl Into<SignKeyPairId>,
) -> AppRes<UserVerifyKeyDataEntity>
{
	let mut out = user_model::get_verify_key_by_id(app_id, user_id, verify_key_id).await?;

	if out.verify_key == "extern" {
		let mut fetched_key = server_key_store::get_keys(&[format!("vk_{}", out.verify_key_id)]).await?;

		if let Some(fetched_key) = fetched_key.remove(&format!("vk_{}", out.verify_key_id)) {
			out.verify_key = fetched_key
		}
	}

	Ok(out)
}

//__________________________________________________________________________________________________
// user fn with jwt

pub async fn init_user(app_data: &AppData, device_id: &str, input: JwtRefreshInput) -> AppRes<UserInitEntity>
{
	//first refresh the user
	let jwt = refresh_jwt(app_data, device_id, input).await?;

	//2nd get all group invites
	let invites = group_user_service::get_invite_req(&app_data.app_data.app_id, &jwt.user_id, 0, "none").await?;

	Ok(UserInitEntity {
		jwt: jwt.jwt,
		invites,
	})
}

pub async fn refresh_jwt(app_data: &AppData, device_id: impl Into<DeviceId>, input: JwtRefreshInput) -> AppRes<DoneLoginLightServerOutput>
{
	let device_id = device_id.into();

	//get the token from the db
	let check = user_model::check_refresh_token(&app_data.app_data.app_id, &device_id, input.refresh_token).await?;

	let device_identifier = match check {
		Some(u) => u,
		None => {
			return Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::RefreshToken,
				"Refresh token not found",
			))
		},
	};

	let jwt = create_jwt(
		&device_identifier.user_id,
		&device_id,
		&app_data.jwt_data[0], //use always the latest created jwt data
		false,
	)
	.await?;

	let out = DoneLoginLightServerOutput {
		user_id: device_identifier.user_id,
		jwt,
		device_id,
	};

	Ok(out)
}

pub async fn delete(user: &UserJwtEntity, app_id: impl Into<AppId>) -> AppRes<()>
{
	let app_id = app_id.into();

	//the user needs a jwt which was created from login and no refreshed jwt
	if !user.fresh {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::WrongJwtAction,
			"The jwt is not valid for this action",
		));
	}

	let user_id = &user.id;
	let group_id = &user.group_id;

	delete_user_external_devices(user_id, &app_id).await?;

	user_model::delete(user_id, &app_id).await?;

	//delete the user in-app check cache from the jwt mw
	let cache_key = get_user_in_app_key(&app_id, user_id);
	cache::delete(&cache_key).await?;

	//delete the user group
	group_service::delete_user_group(app_id, group_id).await
}

async fn delete_user_external_devices(user_id: &str, app_id: &str) -> AppRes<()>
{
	let mut last_id = String::new();

	loop {
		let keys = DeviceForDelete::get_devices_for_external_storage(user_id, app_id, &last_id).await?;

		if keys.is_empty() {
			break;
		}

		//delete the external device keys
		let mut keys_to_delete = vec![];

		for key in &keys {
			if key.public_key == "extern" {
				keys_to_delete.push(format!("sk_{}", key.device_id));
			}

			if key.verify_key == "extern" {
				keys_to_delete.push(format!("vk_{}", key.device_id));
			}

			if key.encrypted_private_key == "extern" {
				keys_to_delete.push(format!("pk_{}", key.device_id));
			}

			if key.encrypted_sign_key == "extern" {
				keys_to_delete.push(format!("sig_pk_{}", key.device_id));
			}
		}

		if !keys_to_delete.is_empty() {
			server_key_store::delete_key(&keys_to_delete).await?;
		}

		if keys.len() < 50 {
			//No more keys to fetch
			break;
		}

		if let Some(last) = keys.last() {
			last_id = last.device_id.clone();
		}
	}

	Ok(())
}

pub async fn delete_device(user: &UserJwtEntity, app_id: impl Into<AppId>, device_id: impl Into<DeviceId>) -> AppRes<()>
{
	let app_id = app_id.into();

	//this can be any device don't need to be the device to delete
	if !user.fresh {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::WrongJwtAction,
			"The jwt is not valid for this action",
		));
	}

	let user_id = &user.id;
	let device_id = device_id.into();

	let keys = DeviceForDelete::get_device_for_external_key_storage(user_id, &app_id, &device_id).await?;

	user_model::delete_device(user_id, &app_id, &device_id).await?;

	group_user_service::leave_group(&internal_group_data(&app_id, &user.group_id, 4), None).await?;

	//delete the external device keys
	let mut keys_to_delete = vec![];

	if keys.public_key == "extern" {
		keys_to_delete.push(format!("sk_{device_id}"));
	}

	if keys.verify_key == "extern" {
		keys_to_delete.push(format!("vk_{device_id}"));
	}

	if keys.encrypted_private_key == "extern" {
		keys_to_delete.push(format!("pk_{device_id}"));
	}

	if keys.encrypted_sign_key == "extern" {
		keys_to_delete.push(format!("sign_k_{device_id}"));
	}

	if !keys_to_delete.is_empty() {
		server_key_store::delete_key(&keys_to_delete).await?;
	}

	Ok(())
}

pub async fn update(user: &UserJwtEntity, app_id: &str, update_input: UserUpdateServerInput) -> AppRes<()>
{
	let user_id = &user.id;

	let identifier = hash_token_to_string(update_input.user_identifier.as_bytes())?;

	//check if the new ident exists
	let exists = user_model::check_user_exists(app_id, &identifier).await?;

	if exists {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::UserExists,
			"User identifier already exists",
		));
	}

	user_model::update(user_id, &user.device_id, app_id, identifier).await
}

pub async fn change_password(user: &UserJwtEntity, app_id: &str, input: ChangePasswordData) -> AppRes<()>
{
	//the user needs a jwt which was created from login and no refreshed jwt
	if !user.fresh {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::WrongJwtAction,
			"The jwt is not valid for this action",
		));
	}

	let user_id = &user.id;
	let device_id = &user.device_id;

	let device_identifier = match user_model::get_device_identifier(app_id, &user.id, &user.device_id).await? {
		Some(i) => i.0,
		None => {
			return Err(ServerCoreError::new_msg(
				401,
				ApiErrorCodes::WrongJwtAction,
				"No device found for this jwt",
			))
		},
	};

	let old_hashed_auth_key = auth_user(app_id, device_identifier.as_str(), input.old_auth_key.to_string()).await?;

	user_model::change_password(user_id, device_id, input, old_hashed_auth_key).await
}

pub async fn delete_all_sessions(user: &UserJwtEntity, app_id: &str) -> AppRes<()>
{
	if !user.fresh {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::WrongJwtAction,
			"The jwt is not valid for this action",
		));
	}

	user_model::delete_all_sessions(&user.id, app_id).await
}

//__________________________________________________________________________________________________
//otp

pub async fn register_otp(app_id: impl Into<AppId>, user_id: impl Into<UserId>) -> AppRes<OtpRegister>
{
	let data = otp::register_otp()?;

	let key = encrypted_at_rest_root::get_key_map().await;

	let encrypted_secret = encrypted_at_rest_root::encrypt_with_key(&key, &data.secret)?;

	//hash the recover tokens for search look up

	let mut encrypted_recover = Vec::with_capacity(6);

	for i in &data.recover {
		encrypted_recover.push((
			encrypted_at_rest_root::encrypt_with_key(&key, i)?,
			hash_token_to_string(i.as_bytes())?,
		))
	}

	user_model::register_otp(app_id, user_id, encrypted_secret, data.alg.clone(), encrypted_recover).await?;

	Ok(data)
}

pub async fn reset_otp(app_id: impl Into<AppId>, user: &UserJwtEntity) -> AppRes<OtpRegister>
{
	if !user.fresh {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::WrongJwtAction,
			"The jwt is not valid for this action",
		));
	}

	let user_id = &user.id;

	//1. delete the old recovery keys
	user_model::delete_all_otp_token(user_id).await?;

	//2. create new secret and new recovery keys
	register_otp(app_id, user_id).await
}

pub async fn disable_otp(user: &UserJwtEntity) -> AppRes<()>
{
	if !user.fresh {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::WrongJwtAction,
			"The jwt is not valid for this action",
		));
	}

	let user_id = &user.id;

	//1. delete the old recovery keys
	user_model::delete_all_otp_token(user_id).await?;

	//2. remove the secret
	user_model::disable_otp(user_id).await
}

pub async fn get_otp_recovery_keys(user: &UserJwtEntity) -> AppRes<OtpRecoveryKeysOutput>
{
	if !user.fresh {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::WrongJwtAction,
			"The jwt is not valid for this action",
		));
	}

	let user_id = &user.id;

	let encrypted_keys = user_model::get_otp_recovery_keys(user_id).await?;

	let key = encrypted_at_rest_root::get_key_map().await;

	let keys = encrypted_keys
		.iter()
		.map(|k| encrypted_at_rest_root::decrypt_with_key(&key, &k.0))
		.collect::<Result<Vec<String>, _>>()?;

	Ok(OtpRecoveryKeysOutput {
		keys,
	})
}

//__________________________________________________________________________________________________
//internal fn

pub(super) fn internal_group_data(app_id: impl Into<AppId>, user_group_id: impl Into<GroupId>, rank: i32) -> InternalGroupDataComplete
{
	InternalGroupDataComplete {
		group_data: InternalGroupData {
			app_id: app_id.into(),
			id: user_group_id.into(),
			time: 0,
			parent: None,
			invite: 1, //must be 1 to accept the device invite
			is_connected_group: false,
		},
		user_data: InternalUserGroupData {
			user_id: "".to_string(),
			real_user_id: "".to_string(),
			joined_time: 0,
			rank,
			get_values_from_parent: None,
			get_values_from_group_as_member: None,
		},
	}
}

pub(super) fn create_refresh_token_raw() -> AppRes<[u8; 50]>
{
	let mut rng = rand::thread_rng();

	let mut token = [0u8; 50];

	rng.try_fill_bytes(&mut token)
		.map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::AppTokenWrongFormat, "Can't create refresh token"))?;

	Ok(token)
}

pub(super) fn create_refresh_token() -> AppRes<String>
{
	let token = create_refresh_token_raw()?;

	Ok(base64::encode_config(token, base64::URL_SAFE_NO_PAD))
}

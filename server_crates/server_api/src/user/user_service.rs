use std::future::Future;
use std::ptr;

use rand::RngCore;
use rustgram_server_util::cache;
use rustgram_server_util::db::StringEntity;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto::util::public::HashedAuthenticationKey;
use sentc_crypto_common::user::{
	ChangePasswordData,
	DoneLoginLightOutput,
	DoneLoginLightServerOutput,
	DoneLoginServerInput,
	JwtRefreshInput,
	PrepareLoginSaltServerOutput,
	RegisterServerOutput,
	ResetPasswordData,
	UserDeviceDoneRegisterInput,
	UserDeviceRegisterInput,
	UserDeviceRegisterOutput,
	UserIdentifierAvailableServerInput,
	UserIdentifierAvailableServerOutput,
	UserUpdateServerInput,
};
use sentc_crypto_common::{AppId, DeviceId, EncryptionKeyPairId, GroupId, SignKeyPairId, SymKeyId, UserId};

use crate::group::group_entities::{GroupUserKeys, InternalGroupData, InternalGroupDataComplete, InternalUserGroupData};
use crate::group::group_user_service::NewUserType;
use crate::group::{group_service, group_user_service, GROUP_TYPE_USER};
use crate::sentc_app_entities::AppData;
use crate::sentc_app_utils::hash_token_to_string;
use crate::sentc_user_entities::{UserPublicKeyDataEntity, UserRegisterData, UserVerifyKeyDataEntity};
use crate::user::jwt::create_jwt;
use crate::user::user_entities::{DoneLoginServerOutput, UserDeviceList, UserInitEntity, UserJwtEntity, SERVER_RANDOM_VALUE};
use crate::user::user_model;
use crate::util::api_res::ApiErrorCodes;
use crate::util::get_user_in_app_key;

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

pub fn save_user_action<'a>(
	app_id: impl Into<String> + 'a,
	user_id: impl Into<String> + 'a,
	action: UserAction,
	amount: i64,
) -> impl Future<Output = AppRes<()>> + 'a
{
	user_model::save_user_action(app_id, user_id, action, amount)
}

pub fn get_group_key_rotations_in_actual_month<'a>(
	app_id: impl Into<AppId> + 'a,
	group_id: impl Into<GroupId> + 'a,
) -> impl Future<Output = AppRes<i32>> + 'a
{
	user_model::get_group_key_rotations_in_actual_month(app_id, group_id)
}

pub fn check_user_in_app_by_user_id<'a>(app_id: impl Into<AppId> + 'a, user_id: impl Into<UserId> + 'a) -> impl Future<Output = AppRes<bool>> + 'a
{
	user_model::check_user_in_app(app_id, user_id)
}

pub fn get_user_group_id<'a>(
	app_id: impl Into<AppId> + 'a,
	user_id: impl Into<UserId> + 'a,
) -> impl Future<Output = AppRes<Option<StringEntity>>> + 'a
{
	user_model::get_user_group_id(app_id, user_id)
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

pub async fn register_light(app_id: impl Into<AppId>, input: UserDeviceRegisterInput) -> AppRes<(String, String)>
{
	let app_id = app_id.into();

	let identifier = hash_token_to_string(input.device_identifier.as_bytes())?;

	let (user_id, device_id) = user_model::register(&app_id, identifier, input.master_key, input.derived).await?;

	//delete the user in app check cache from the jwt mw
	//it can happened that a user id was used before which doesn't exists yet
	let cache_key = get_user_in_app_key(&app_id, &user_id);
	cache::delete(&cache_key).await?;

	Ok((user_id, device_id))
}

pub async fn register(app_id: impl Into<AppId>, register_input: UserRegisterData) -> AppRes<RegisterServerOutput>
{
	let mut group_data = register_input.group;

	if group_data.public_key_sig.is_none() ||
		group_data.verify_key.is_none() ||
		group_data.encrypted_sign_key.is_none() ||
		group_data.keypair_sign_alg.is_none()
	{
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::UserKeysNotFound,
			"User keys not found. Make sure to create the user group.",
		));
	}

	let device_data = register_input.device;

	let app_id = app_id.into();

	//save the data

	let identifier = hash_token_to_string(device_data.device_identifier.as_bytes())?;

	let (user_id, device_id) = user_model::register(&app_id, identifier, device_data.master_key, device_data.derived).await?;

	//update creator public key id in group data (with the device id), this is needed to know what public key was used to encrypt the group key
	group_data.creator_public_key_id = device_id.to_string();

	//create user group, insert the device not the suer id because the devices are in the group not the user!
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
	//it can happened that a user id was used before which doesn't exists yet
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

	let check = user_model::check_user_exists(&app_id, &input.device_identifier).await?;

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

	let identifier = hash_token_to_string(input.device_identifier.as_bytes())?;

	let token = create_refresh_token()?;

	let device_id = user_model::register_device(app_id, identifier, input.master_key, input.derived, &token).await?;

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
		&InternalGroupDataComplete {
			group_data: InternalGroupData {
				app_id: app_id.clone(),
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
				rank: 0, //Rank must be 0
				get_values_from_parent: None,
				get_values_from_group_as_member: None,
			},
		},
		input.user_keys,
		&device_id, //invite the new device
		NewUserType::Normal,
	)
	.await?;

	user_model::done_register_device(app_id, user_id, device_id).await?;

	Ok(session_id)
}

//__________________________________________________________________________________________________

pub fn prepare_login<'a>(app_data: &'a AppData, user_identifier: &'a str) -> impl Future<Output = AppRes<PrepareLoginSaltServerOutput>> + 'a
{
	create_salt(&app_data.app_data.app_id, user_identifier)
}

/**
Only the jwt and user id, no keys
*/
pub async fn done_login_light(app_data: &AppData, done_login: DoneLoginServerInput) -> AppRes<DoneLoginLightOutput>
{
	let identifier = hash_token_to_string(done_login.device_identifier.as_bytes())?;

	auth_user(&app_data.app_data.app_id, &identifier, done_login.auth_key).await?;

	let id = user_model::get_done_login_light_data(app_data.app_data.app_id.as_str(), identifier)
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(401, ApiErrorCodes::Login, "Wrong username or password"))?;

	let jwt = create_jwt(
		&id.user_id,
		&id.device_id,
		&app_data.jwt_data[0], //use always the latest created jwt data
		true,
	)
	.await?;

	let refresh_token = create_refresh_token()?;

	//activate refresh token
	user_model::insert_refresh_token(&app_data.app_data.app_id, &id.device_id, &refresh_token).await?;

	let out = DoneLoginLightOutput {
		user_id: id.user_id,
		jwt,
		device_id: id.device_id,
		refresh_token,
	};

	Ok(out)
}

/**
After successful login return the user keys so they can be decrypted in the client
*/
pub async fn done_login(app_data: &AppData, done_login: DoneLoginServerInput) -> AppRes<DoneLoginServerOutput>
{
	let identifier = hash_token_to_string(done_login.device_identifier.as_bytes())?;

	auth_user(&app_data.app_data.app_id, &identifier, done_login.auth_key).await?;

	//if correct -> fetch and return the user data
	let device_keys = user_model::get_done_login_data(&app_data.app_data.app_id, identifier)
		.await?
		.ok_or_else(|| ServerCoreError::new_msg(401, ApiErrorCodes::Login, "Wrong username or password"))?;

	// and create the jwt
	let jwt = create_jwt(
		&device_keys.user_id,
		&device_keys.device_id,
		&app_data.jwt_data[0], //use always the latest created jwt data
		true,
	)
	.await?;

	let refresh_token = create_refresh_token()?;

	//activate refresh token
	user_model::insert_refresh_token(&app_data.app_data.app_id, &device_keys.device_id, &refresh_token).await?;

	//fetch the first page of the group keys with the device id as user
	let user_keys = group_service::get_user_group_keys(
		&app_data.app_data.app_id,
		&device_keys.user_group_id,
		&device_keys.device_id,
		0,
		"",
	)
	.await?;

	let hmac_keys = group_service::get_group_hmac(
		&app_data.app_data.app_id,
		&device_keys.user_group_id,
		0, //fetch the first page
		"",
	)
	.await?;

	//fetch the first page of the hmac keys

	let out = DoneLoginServerOutput {
		device_keys,
		user_keys,
		hmac_keys,
		jwt,
		refresh_token,
	};

	Ok(out)
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

//__________________________________________________________________________________________________
//public user fn

pub fn get_public_key_by_id<'a>(
	app_id: impl Into<AppId> + 'a,
	user_id: impl Into<UserId> + 'a,
	public_key_id: impl Into<EncryptionKeyPairId> + 'a,
) -> impl Future<Output = AppRes<UserPublicKeyDataEntity>> + 'a
{
	user_model::get_public_key_by_id(app_id, user_id, public_key_id)
}

pub fn get_public_key_data<'a>(
	app_id: impl Into<AppId> + 'a,
	user_id: impl Into<UserId> + 'a,
) -> impl Future<Output = AppRes<UserPublicKeyDataEntity>> + 'a
{
	user_model::get_public_key_data(app_id, user_id)
}

pub fn get_verify_key_by_id<'a>(
	app_id: impl Into<AppId> + 'a,
	user_id: impl Into<UserId> + 'a,
	verify_key_id: impl Into<SignKeyPairId> + 'a,
) -> impl Future<Output = AppRes<UserVerifyKeyDataEntity>> + 'a
{
	user_model::get_verify_key_by_id(app_id, user_id, verify_key_id)
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

	user_model::delete(user_id, &app_id).await?;

	//delete the user in app check cache from the jwt mw
	let cache_key = get_user_in_app_key(&app_id, user_id);
	cache::delete(&cache_key).await?;

	//delete the user group
	group_service::delete_user_group(app_id, group_id).await?;

	Ok(())
}

pub fn delete_from_server<'a>(user_id: impl Into<UserId> + 'a, app_id: impl Into<AppId> + 'a) -> impl Future<Output = AppRes<()>> + 'a
{
	user_model::delete(user_id, app_id)
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

	user_model::delete_device(user_id, &app_id, device_id).await?;

	group_user_service::leave_group(
		&InternalGroupDataComplete {
			group_data: InternalGroupData {
				app_id,
				id: user.group_id.to_string(),
				time: 0,
				parent: None,
				invite: 0,
				is_connected_group: false,
			},
			user_data: InternalUserGroupData {
				user_id: user_id.to_string(),
				real_user_id: "".to_string(),
				joined_time: 0,
				rank: 4,
				get_values_from_parent: None,
				get_values_from_group_as_member: None,
			},
		},
		None,
	)
	.await
}

pub fn get_devices<'a>(
	app_id: impl Into<AppId> + 'a,
	user_id: impl Into<UserId> + 'a,
	last_fetched_time: u128,
	last_fetched_id: impl Into<DeviceId> + 'a,
) -> impl Future<Output = AppRes<Vec<UserDeviceList>>> + 'a
{
	user_model::get_devices(app_id, user_id, last_fetched_time, last_fetched_id)
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

	user_model::update(user_id, &user.device_id, app_id, identifier).await?;

	Ok(())
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

	user_model::change_password(user_id, device_id, input, old_hashed_auth_key).await?;

	Ok(())
}

pub fn reset_password<'a>(
	user_id: impl Into<UserId> + 'a,
	device_id: impl Into<DeviceId> + 'a,
	input: ResetPasswordData,
) -> impl Future<Output = AppRes<()>> + 'a
{
	//no fresh jwt here because the user can't login and get a fresh jwt without the password
	//but still needs a valid jwt. jwt refresh is possible without a password!

	user_model::reset_password(user_id, device_id, input)
}

//__________________________________________________________________________________________________
//internal fn

async fn create_salt(app_id: impl Into<AppId>, user_identifier: &str) -> AppRes<PrepareLoginSaltServerOutput>
{
	let identifier = hash_token_to_string(user_identifier.as_bytes())?;

	//check the user id in the db
	let login_data = user_model::get_user_login_data(app_id, &identifier).await?;

	//create the salt
	let (client_random_value, alg, add_str) = match login_data {
		Some(d) => (d.client_random_value, d.derived_alg, ""),

		//when user_identifier not found, push the user_identifier to the salt string, use the default alg
		None => {
			(
				SERVER_RANDOM_VALUE.0.to_owned(),
				SERVER_RANDOM_VALUE.1.to_owned(),
				user_identifier,
			)
		},
	};

	let salt_string = sentc_crypto::util::server::generate_salt_from_base64_to_string(client_random_value.as_str(), alg.as_str(), add_str)
		.map_err(|_e| ServerCoreError::new_msg(401, ApiErrorCodes::SaltError, "Can't create salt"))?;

	let out = PrepareLoginSaltServerOutput {
		salt_string,
		derived_encryption_key_alg: alg,
	};

	Ok(out)
}

fn create_refresh_token() -> AppRes<String>
{
	let mut rng = rand::thread_rng();

	let mut token = [0u8; 50];

	rng.try_fill_bytes(&mut token)
		.map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::AppTokenWrongFormat, "Can't create refresh token"))?;

	let token_string = base64::encode_config(token, base64::URL_SAFE_NO_PAD);

	Ok(token_string)
}

async fn auth_user(app_id: &str, hashed_user_identifier: impl Into<String>, auth_key: String) -> AppRes<String>
{
	//get the login data
	let login_data = user_model::get_user_login_data(app_id, hashed_user_identifier).await?;

	let (hashed_user_auth_key, alg) = match login_data {
		Some(d) => (d.hashed_authentication_key, d.derived_alg),
		None => {
			return Err(ServerCoreError::new_msg(
				401,
				ApiErrorCodes::UserNotFound,
				"No user found with this identifier",
			))
		},
	};

	//hash the auth key and use the first 16 bytes
	let (server_hashed_auth_key, hashed_client_key) =
		sentc_crypto::util::server::get_auth_keys_from_base64(auth_key.as_str(), hashed_user_auth_key.as_str(), alg.as_str()).map_err(|_e| {
			ServerCoreError::new_msg(
				401,
				ApiErrorCodes::AuthKeyFormat,
				"The authentication key has a wrong format",
			)
		})?;

	//check the keys
	let check = compare_auth_keys(server_hashed_auth_key, hashed_client_key);

	//if not correct -> err msg
	if !check {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::Login,
			"Wrong username or password",
		));
	}

	//return this here for the update user pw functions
	Ok(hashed_user_auth_key)
}

/// Secure `memeq`.
/// from here:https://github.com/quininer/memsec/blob/master/src/lib.rs#L22
#[inline(never)]
unsafe fn memeq(b1: *const u8, b2: *const u8, len: usize) -> bool
{
	(0..len)
		.map(|i| ptr::read_volatile(b1.add(i)) ^ ptr::read_volatile(b2.add(i)))
		.fold(0, |sum, next| sum | next)
		.eq(&0)
}

fn compare_auth_keys(left: HashedAuthenticationKey, right: HashedAuthenticationKey) -> bool
{
	match (left, right) {
		(HashedAuthenticationKey::Argon2(l), HashedAuthenticationKey::Argon2(r)) => {
			//calling in unsafe block

			unsafe {
				//
				memeq(l.as_ptr(), r.as_ptr(), 16)
			}
		},
	}
}

use std::future::Future;
use std::ptr;

use rand::RngCore;
use sentc_crypto::sdk_common::GroupId;
use sentc_crypto::util::public::HashedAuthenticationKey;
use sentc_crypto_common::group::GroupKeysForNewMemberServerInput;
use sentc_crypto_common::user::{
	ChangePasswordData,
	DoneLoginLightServerOutput,
	DoneLoginServerInput,
	JwtRefreshInput,
	PrepareLoginSaltServerOutput,
	PrepareLoginServerInput,
	RegisterData,
	RegisterServerOutput,
	ResetPasswordData,
	UserDeviceRegisterInput,
	UserDeviceRegisterOutput,
	UserIdentifierAvailableServerInput,
	UserIdentifierAvailableServerOutput,
	UserUpdateServerInput,
};
use sentc_crypto_common::{AppId, DeviceId, UserId};

use crate::group::group_entities::{InternalGroupData, InternalGroupDataComplete, InternalUserGroupData};
use crate::group::{group_service, group_user_service, GROUP_TYPE_USER};
use crate::user::jwt::create_jwt;
use crate::user::user_entities::{DoneLoginServerOutput, UserInitEntity, UserJwtEntity, SERVER_RANDOM_VALUE};
use crate::user::user_model;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::AppData;

pub fn check_user_in_app_by_user_id(app_id: AppId, user_id: UserId) -> impl Future<Output = AppRes<bool>>
{
	user_model::check_user_in_app(app_id, user_id)
}

pub async fn exists(app_data: &AppData, data: UserIdentifierAvailableServerInput) -> AppRes<UserIdentifierAvailableServerOutput>
{
	let exists = user_model::check_user_exists(&app_data.app_data.app_id, data.user_identifier.as_str()).await?;

	let out = UserIdentifierAvailableServerOutput {
		user_identifier: data.user_identifier,
		available: !exists,
	};

	Ok(out)
}

pub async fn register(app_id: AppId, register_input: RegisterData) -> AppRes<RegisterServerOutput>
{
	let mut group_data = register_input.group;
	let device_data = register_input.device;

	let device_identifier = device_data.device_identifier.to_string(); //save this value before because of dropping

	//save the data
	let (user_id, device_id) = user_model::register(app_id.to_string(), device_data).await?;

	//update creator public key id in group data (with the device id), this is needed to know what public key was used to encrypt the group key
	group_data.creator_public_key_id = device_id.to_string();

	//create user group, insert the device not the suer id because the devices are in the group not the user!
	let group_id = group_service::create_group(
		app_id.to_string(),
		device_id.to_string(),
		group_data,
		GROUP_TYPE_USER,
		None,
		None,
	)
	.await?;

	//now update the user group id
	user_model::register_update_user_group_id(app_id, user_id.to_string(), group_id).await?;

	let out = RegisterServerOutput {
		user_id,
		device_id,
		device_identifier,
	};

	Ok(out)
}

pub async fn prepare_register_device(app_id: AppId, user_id: UserId, input: UserDeviceRegisterInput) -> AppRes<UserDeviceRegisterOutput>
{
	//TODO endpoint

	let device_identifier = input.device_identifier.to_string();
	let device_id = user_model::register_device(app_id.to_string(), input, user_id).await?;

	Ok(UserDeviceRegisterOutput {
		device_id,
		device_identifier,
	})
}

pub async fn done_register_device(
	app_id: AppId,
	user_id: UserId,
	user_group_id: GroupId,
	device_id: DeviceId,
	input: GroupKeysForNewMemberServerInput,
) -> AppRes<Option<String>>
{
	//TODO endpoint

	//for the auto invite we only need the group id and the group user rank
	let session_id = group_user_service::invite_auto(
		&InternalGroupDataComplete {
			group_data: InternalGroupData {
				app_id,
				id: user_group_id,
				time: 0,
				parent: None,
			},
			user_data: InternalUserGroupData {
				user_id: user_id.to_string(),
				real_user_id: user_id,
				joined_time: 0,
				rank: 0, //Rank must be 0
				get_values_from_parent: None,
			},
		},
		input,
		device_id.to_string(), //invite the new device
	)
	.await?;

	Ok(session_id)
}

//__________________________________________________________________________________________________

pub async fn prepare_login(app_data: &AppData, user_identifier: PrepareLoginServerInput) -> AppRes<PrepareLoginSaltServerOutput>
{
	let out = create_salt(&app_data.app_data.app_id, user_identifier.user_identifier.as_str()).await?;

	Ok(out)
}

/**
Only the jwt and user id, no keys
*/
pub async fn done_login_light(app_data: &AppData, done_login: DoneLoginServerInput, aud: &str) -> AppRes<DoneLoginLightServerOutput>
{
	auth_user(
		app_data.app_data.app_id.to_string(),
		done_login.device_identifier.as_str(),
		done_login.auth_key,
	)
	.await?;

	let id = user_model::get_done_login_light_data(
		app_data.app_data.app_id.as_str(),
		done_login.device_identifier.as_str(),
	)
	.await?;

	let id = match id {
		Some(d) => d,
		None => {
			return Err(HttpErr::new(
				401,
				ApiErrorCodes::Login,
				"Wrong username or password".to_owned(),
				None,
			))
		},
	};

	let jwt = create_jwt(
		id.user_id.to_string(),
		id.group_id,
		id.device_id.to_string(),
		done_login.device_identifier,
		app_data.app_data.app_id.as_str(),
		&app_data.jwt_data[0], //use always the latest created jwt data
		aud,
		true,
	)
	.await?;

	let out = DoneLoginLightServerOutput {
		user_id: id.user_id,
		jwt,
		device_id: id.device_id,
	};

	Ok(out)
}

/**
After successful login return the user keys so they can be decrypted in the client
*/
pub async fn done_login(app_data: &AppData, done_login: DoneLoginServerInput) -> AppRes<DoneLoginServerOutput>
{
	auth_user(
		app_data.app_data.app_id.to_string(),
		done_login.device_identifier.as_str(),
		done_login.auth_key,
	)
	.await?;

	//if correct -> fetch and return the user data
	let device_keys = user_model::get_done_login_data(&app_data.app_data.app_id, done_login.device_identifier.as_str()).await?;

	let device_keys = match device_keys {
		Some(d) => d,
		None => {
			return Err(HttpErr::new(
				401,
				ApiErrorCodes::Login,
				"Wrong username or password".to_owned(),
				None,
			))
		},
	};

	// and create the jwt
	let jwt = create_jwt(
		device_keys.user_id.to_string(),
		device_keys.user_group_id.to_string(),
		device_keys.device_id.to_string(),
		done_login.device_identifier,
		app_data.app_data.app_id.as_str(),
		&app_data.jwt_data[0], //use always the latest created jwt data
		"user",
		true,
	)
	.await?;

	let refresh_token = create_refresh_token()?;

	//activate refresh token
	user_model::insert_refresh_token(
		app_data.app_data.app_id.to_string(),
		device_keys.device_id.to_string(),
		refresh_token.to_string(),
	)
	.await?;

	//fetch the first page of the group keys with the device id as user
	let user_keys = group_service::get_user_group_keys(
		app_data.app_data.app_id.to_string(),
		device_keys.user_group_id.to_string(),
		device_keys.device_id.to_string(),
		0,
		"".to_string(),
	)
	.await?;

	let out = DoneLoginServerOutput {
		device_keys,
		user_keys,
		jwt,
		refresh_token,
	};

	Ok(out)
}

//__________________________________________________________________________________________________
// user fn with jwt

pub async fn init_user(app_data: &AppData, device_id: DeviceId, input: JwtRefreshInput) -> AppRes<UserInitEntity>
{
	//first refresh the user
	let jwt = refresh_jwt(app_data, device_id.to_string(), input, "user").await?;

	//2nd get all group invites
	let invites = group_user_service::get_invite_req(
		app_data.app_data.app_id.to_string(),
		jwt.user_id,
		0,
		"none".to_string(),
	)
	.await?;

	Ok(UserInitEntity {
		jwt: jwt.jwt,
		invites,
	})
}

pub async fn refresh_jwt(app_data: &AppData, device_id: DeviceId, input: JwtRefreshInput, aud: &str) -> AppRes<DoneLoginLightServerOutput>
{
	//get the token from the db
	let check = user_model::check_refresh_token(
		app_data.app_data.app_id.to_string(),
		device_id.to_string(),
		input.refresh_token,
	)
	.await?;

	let device_identifier = match check {
		Some(u) => u,
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::RefreshToken,
				"Refresh token not found".to_string(),
				None,
			))
		},
	};

	let jwt = create_jwt(
		device_identifier.user_id.to_string(),
		device_identifier.group_id,
		device_id.to_string(),
		device_identifier.device_identifier,
		app_data.app_data.app_id.as_str(),
		&app_data.jwt_data[0], //use always the latest created jwt data
		aud,
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

pub async fn delete(user: &UserJwtEntity) -> AppRes<()>
{
	//the user needs a jwt which was created from login and no refreshed jwt
	if user.fresh == false {
		return Err(HttpErr::new(
			401,
			ApiErrorCodes::WrongJwtAction,
			"The jwt is not valid for this action".to_string(),
			None,
		));
	}

	let user_id = &user.id;
	let app_id = &user.sub.to_string();

	user_model::delete(user_id.to_string(), app_id.to_string()).await?;

	group_service::delete_user_group(app_id.to_string(), user_id.to_string()).await?;

	Ok(())
}

pub async fn delete_device(user: &UserJwtEntity, device_id: DeviceId) -> AppRes<()>
{
	//this can be any device don't need to be the device to delete
	if user.fresh == false {
		return Err(HttpErr::new(
			401,
			ApiErrorCodes::WrongJwtAction,
			"The jwt is not valid for this action".to_string(),
			None,
		));
	}

	let user_id = &user.id;
	let app_id = &user.sub.to_string();

	user_model::delete_device(user_id.to_string(), app_id.to_string(), device_id).await
}

pub async fn update(user: &UserJwtEntity, update_input: UserUpdateServerInput) -> AppRes<()>
{
	let user_id = &user.id;
	let app_id = &user.sub;

	//check if the new ident exists
	let exists = user_model::check_user_exists(app_id, update_input.user_identifier.as_str()).await?;

	if exists {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::UserExists,
			"User identifier already exists".to_string(),
			None,
		));
	}

	user_model::update(
		user_id.to_string(),
		user.device_id.to_string(),
		app_id.to_string(),
		update_input.user_identifier,
	)
	.await?;

	Ok(())
}

pub async fn change_password(user: &UserJwtEntity, input: ChangePasswordData) -> AppRes<()>
{
	//the user needs a jwt which was created from login and no refreshed jwt
	if user.fresh == false {
		return Err(HttpErr::new(
			401,
			ApiErrorCodes::WrongJwtAction,
			"The jwt is not valid for this action".to_string(),
			None,
		));
	}

	let user_id = &user.id;
	let user_identifier = &user.identifier;
	let app_id = &user.sub;
	let device_id = &user.device_id;

	let old_hashed_auth_key = auth_user(app_id.to_string(), user_identifier, input.old_auth_key.to_string()).await?;

	user_model::change_password(user_id.to_string(), device_id.to_string(), input, old_hashed_auth_key).await?;

	Ok(())
}

pub fn reset_password(user_id: UserId, device_id: String, input: ResetPasswordData) -> impl Future<Output = AppRes<()>>
{
	//no fresh jwt here because the user can't login and get a fresh jwt without the password
	//but still needs a valid jwt. jwt refresh is possible without a password!

	user_model::reset_password(user_id, device_id, input)
}

//__________________________________________________________________________________________________
//internal fn

async fn create_salt(app_id: &str, user_identifier: &str) -> AppRes<PrepareLoginSaltServerOutput>
{
	//check the user id in the db
	let login_data = user_model::get_user_login_data(app_id.to_string(), user_identifier).await?;

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
		.map_err(|_e| HttpErr::new(401, ApiErrorCodes::SaltError, "Can't create salt".to_owned(), None))?;

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

	rng.try_fill_bytes(&mut token).map_err(|_| {
		HttpErr::new(
			400,
			ApiErrorCodes::AppTokenWrongFormat,
			"Can't create refresh token".to_string(),
			None,
		)
	})?;

	let token_string = base64::encode_config(token, base64::URL_SAFE_NO_PAD);

	Ok(token_string)
}

async fn auth_user(app_id: AppId, user_identifier: &str, auth_key: String) -> AppRes<String>
{
	//get the login data
	let login_data = user_model::get_user_login_data(app_id, user_identifier).await?;

	let (hashed_user_auth_key, alg) = match login_data {
		Some(d) => (d.hashed_authentication_key, d.derived_alg),
		None => {
			return Err(HttpErr::new(
				401,
				ApiErrorCodes::UserNotFound,
				"No user found with this identifier".to_string(),
				None,
			))
		},
	};

	//hash the auth key and use the first 16 bytes
	let (server_hashed_auth_key, hashed_client_key) =
		sentc_crypto::util::server::get_auth_keys_from_base64(auth_key.as_str(), hashed_user_auth_key.as_str(), alg.as_str()).map_err(|_e| {
			HttpErr::new(
				401,
				ApiErrorCodes::AuthKeyFormat,
				"The authentication key has a wrong format".to_owned(),
				None,
			)
		})?;

	//check the keys
	let check = compare_auth_keys(server_hashed_auth_key, hashed_client_key);

	//if not correct -> err msg
	if !check {
		return Err(HttpErr::new(
			401,
			ApiErrorCodes::Login,
			"Wrong username or password".to_owned(),
			None,
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

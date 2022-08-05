use std::ptr;

use sentc_crypto::util_pub::HashedAuthenticationKey;
use sentc_crypto_common::user::{
	ChangePasswordData,
	DoneLoginServerInput,
	DoneLoginServerKeysOutput,
	PrepareLoginSaltServerOutput,
	PrepareLoginServerInput,
	RegisterData,
	RegisterServerOutput,
	ResetPasswordData,
	UserIdentifierAvailableServerInput,
	UserIdentifierAvailableServerOutput,
	UserUpdateServerInput,
	UserUpdateServerOut,
};
use sentc_crypto_common::AppId;

use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::user::jwt::create_jwt;
use crate::user::user_entities::{UserJwtEntity, SERVER_RANDOM_VALUE};
use crate::user::user_model;
use crate::AppData;

pub async fn exists(app_data: &AppData, data: UserIdentifierAvailableServerInput) -> AppRes<UserIdentifierAvailableServerOutput>
{
	let exists = user_model::check_user_exists(&app_data.app_data.app_id, data.user_identifier.as_str()).await?;

	let out = UserIdentifierAvailableServerOutput {
		user_identifier: data.user_identifier,
		available: exists,
	};

	Ok(out)
}

pub async fn register(app_data: &AppData, register_input: RegisterData) -> AppRes<RegisterServerOutput>
{
	let user_identifier = register_input.user_identifier.to_string(); //save this value before because of dropping

	//save the data
	let user_id = user_model::register(&app_data.app_data.app_id, register_input).await?;

	let out = RegisterServerOutput {
		user_id,
		user_identifier,
	};

	Ok(out)
}

pub async fn prepare_login(app_data: &AppData, user_identifier: PrepareLoginServerInput) -> AppRes<PrepareLoginSaltServerOutput>
{
	let out = create_salt(&app_data.app_data.app_id, user_identifier.user_identifier.as_str()).await?;

	Ok(out)
}

pub async fn done_login(app_data: &AppData, done_login: DoneLoginServerInput) -> AppRes<DoneLoginServerKeysOutput>
{
	auth_user(
		app_data.app_data.app_id.to_string(),
		done_login.user_identifier.as_str(),
		done_login.auth_key,
	)
	.await?;

	//if correct -> fetch and return the user data
	let user_data = user_model::get_done_login_data(&app_data.app_data.app_id, done_login.user_identifier.as_str()).await?;

	let user_data = match user_data {
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
		user_data.user_id.as_str(),
		done_login.user_identifier.as_str(),
		app_data.app_data.app_id.as_str(),
		&app_data.jwt_data[0], //use always the latest created jwt data
		"user",
		true,
	)
	.await?;

	let out = DoneLoginServerKeysOutput {
		encrypted_master_key: user_data.encrypted_master_key,
		encrypted_private_key: user_data.encrypted_private_key,
		public_key_string: user_data.public_key_string,
		keypair_encrypt_alg: user_data.keypair_encrypt_alg,
		encrypted_sign_key: user_data.encrypted_sign_key,
		verify_key_string: user_data.verify_key_string,
		keypair_sign_alg: user_data.keypair_sign_alg,
		keypair_encrypt_id: user_data.keypair_encrypt_id,
		keypair_sign_id: user_data.keypair_sign_id,
		jwt,
		user_id: user_data.user_id,
	};

	Ok(out)
}

//__________________________________________________________________________________________________
// user fn with jwt

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

	user_model::delete(user_id, app_id.to_string()).await?;

	Ok(())
}

pub async fn update(user: &UserJwtEntity, update_input: UserUpdateServerInput) -> AppRes<UserUpdateServerOut>
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

	user_model::update(user_id, app_id.to_string(), update_input.user_identifier.as_str()).await?;

	let out = UserUpdateServerOut {
		user_identifier: update_input.user_identifier,
		user_id: user_id.to_string(),
		msg: "User updated".to_string(),
	};

	Ok(out)
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

	let old_hashed_auth_key = auth_user(app_id.to_string(), user_identifier, input.old_auth_key.to_string()).await?;

	user_model::change_password(user_id, input, old_hashed_auth_key).await?;

	Ok(())
}

pub async fn reset_password(user_id: &str, input: ResetPasswordData) -> AppRes<()>
{
	//no fresh jwt here because the user can't login and get a fresh jwt without the password
	//but still needs a valid jwt. jwt refresh is possible without a password!

	user_model::reset_password(user_id, input).await?;

	Ok(())
}

//__________________________________________________________________________________________________
//internal fn

async fn create_salt(app_id: &str, user_identifier: &str) -> Result<PrepareLoginSaltServerOutput, HttpErr>
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

	let salt_string = sentc_crypto::util_server::generate_salt_from_base64_to_string(client_random_value.as_str(), alg.as_str(), add_str)
		.map_err(|_e| HttpErr::new(401, ApiErrorCodes::SaltError, "Can't create salt".to_owned(), None))?;

	let out = PrepareLoginSaltServerOutput {
		salt_string,
		derived_encryption_key_alg: alg,
	};

	Ok(out)
}

async fn auth_user(app_id: AppId, user_identifier: &str, auth_key: String) -> Result<String, HttpErr>
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
		sentc_crypto::util_server::get_auth_keys_from_base64(auth_key.as_str(), hashed_user_auth_key.as_str(), alg.as_str()).map_err(|_e| {
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

use ring::digest::{Context, SHA256};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;

use crate::ApiErrorCodes;

pub const JWT_CACHE: &str = "jwtcache_";
pub const APP_TOKEN_CACHE: &str = "apptokencache_";
pub const INTERNAL_GROUP_DATA_CACHE: &str = "internalgroupdatacache_";
pub const INTERNAL_GROUP_USER_DATA_CACHE: &str = "internalgroupuserdatacache_";
pub const INTERNAL_GROUP_USER_PARENT_REF_CACHE: &str = "internalgroupuserparentrefcache_";
pub const APP_JWT_VERIFY_KEY_CACHE: &str = "appjwtverifykeycache_";
pub const APP_JWT_SIGN_KEY_CACHE: &str = "appjwtsignkeycache_";
pub const USER_IN_APP_CACHE: &str = "userinappcache_";

pub fn get_group_cache_key(app_id: &str, group_id: &str) -> String
{
	INTERNAL_GROUP_DATA_CACHE.to_string() + app_id + "_" + group_id
}

pub fn get_group_user_cache_key(app_id: &str, group_id: &str, user_id: &str) -> String
{
	INTERNAL_GROUP_USER_DATA_CACHE.to_string() + app_id + "_" + group_id + "_" + user_id
}

pub fn get_group_user_parent_ref_key(group_id: &str, user_id: &str) -> String
{
	INTERNAL_GROUP_USER_PARENT_REF_CACHE.to_string() + group_id + "_" + user_id
}

pub fn get_user_jwt_key(app_id: &str, jwt_key: &str) -> String
{
	JWT_CACHE.to_string() + app_id + "_" + jwt_key
}

pub fn get_app_jwt_verify_key(key_id: &str) -> String
{
	APP_JWT_VERIFY_KEY_CACHE.to_string() + key_id
}

pub fn get_app_jwt_sign_key(key_id: &str) -> String
{
	APP_JWT_SIGN_KEY_CACHE.to_string() + key_id
}

pub fn get_user_in_app_key(app_id: &str, user_id: &str) -> String
{
	USER_IN_APP_CACHE.to_string() + app_id + "_" + user_id
}

pub const HASH_ALG: &str = "SHA256";

pub fn hash_token(token: &[u8]) -> AppRes<[u8; 32]>
{
	let mut context = Context::new(&SHA256);
	context.update(token);
	let result = context.finish();

	let hashed_token: [u8; 32] = result
		.as_ref()
		.try_into()
		.map_err(|_e| ServerCoreError::new_msg(400, ApiErrorCodes::AppTokenWrongFormat, "Token can't be hashed"))?;

	Ok(hashed_token)
}

pub fn hash_token_to_string(token: &[u8]) -> AppRes<String>
{
	let token = hash_token(token)?;

	Ok(base64::encode(token))
}

pub fn hash_token_from_string_to_string(token: &str) -> AppRes<String>
{
	//the normal token is also encoded as base64 when exporting it to user
	let token = base64::decode(token).map_err(|_e| ServerCoreError::new_msg(401, ApiErrorCodes::AppTokenWrongFormat, "Token can't be hashed"))?;

	hash_token_to_string(&token)
}

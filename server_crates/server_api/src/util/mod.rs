pub mod api_res;

pub const JWT_CACHE: &str = "jwtcache_";
pub const APP_TOKEN_CACHE: &str = "apptokencache_";
pub const INTERNAL_GROUP_DATA_CACHE: &str = "internalgroupdatacache_";
pub const INTERNAL_GROUP_USER_DATA_CACHE: &str = "internalgroupuserdatacache_";
pub const INTERNAL_GROUP_USER_PARENT_REF_CACHE: &str = "internalgroupuserparentrefcache_";
pub const APP_JWT_VERIFY_KEY_CACHE: &str = "appjwtverifykeycache_";
pub const APP_JWT_SIGN_KEY_CACHE: &str = "appjwtsignkeycache_";
pub const USER_IN_APP_CACHE: &str = "userinappcache_";

pub(crate) fn get_group_cache_key(app_id: &str, group_id: &str) -> String
{
	INTERNAL_GROUP_DATA_CACHE.to_string() + app_id + "_" + group_id
}

pub(crate) fn get_group_user_cache_key(app_id: &str, group_id: &str, user_id: &str) -> String
{
	INTERNAL_GROUP_USER_DATA_CACHE.to_string() + app_id + "_" + group_id + "_" + user_id
}

pub(crate) fn get_group_user_parent_ref_key(group_id: &str, user_id: &str) -> String
{
	INTERNAL_GROUP_USER_PARENT_REF_CACHE.to_string() + group_id + "_" + user_id
}

pub(crate) fn get_user_jwt_key(app_id: &str, jwt_key: &str) -> String
{
	JWT_CACHE.to_string() + app_id + "_" + jwt_key
}

pub(crate) fn get_app_jwt_verify_key(key_id: &str) -> String
{
	APP_JWT_VERIFY_KEY_CACHE.to_string() + key_id
}

pub(crate) fn get_app_jwt_sign_key(key_id: &str) -> String
{
	APP_JWT_SIGN_KEY_CACHE.to_string() + key_id
}

pub(crate) fn get_user_in_app_key(app_id: &str, user_id: &str) -> String
{
	USER_IN_APP_CACHE.to_string() + app_id + "_" + user_id
}

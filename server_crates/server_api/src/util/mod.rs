pub mod api_res;

pub const JWT_CACHE: &str = "jwtcache_";
pub const APP_TOKEN_CACHE: &str = "apptokencache_";
pub const INTERNAL_GROUP_DATA_CACHE: &str = "internalgroupdatacache_";
pub const INTERNAL_GROUP_USER_DATA_CACHE: &str = "internalgroupuserdatacache_";
pub const INTERNAL_GROUP_USER_PARENT_REF_CACHE: &str = "internalgroupuserparentrefcache_";

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

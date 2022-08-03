pub static JWT_CACHE: &'static str = "jwtcache_";
pub static APP_TOKEN_CACHE: &'static str = "apptokencache_";
pub static INTERNAL_GROUP_DATA_CACHE: &'static str = "internalgroupdatacache_";
pub static INTERNAL_GROUP_USER_DATA_CACHE: &'static str = "internalgroupuserdatacache_";
pub static INTERNAL_GROUP_USER_PARENT_REF_CACHE: &'static str = "internalgroupuserparentrefcache_";

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

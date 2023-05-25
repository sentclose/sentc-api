use chrono::{Datelike, TimeZone, Utc};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use uuid::{Uuid, Version};

use crate::util::api_res::ApiErrorCodes;

pub mod api_res;
pub mod email;

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

pub(crate) fn get_begin_of_month() -> AppRes<i64>
{
	let current_date = Utc::now();

	// Create a new DateTime representing the beginning of the current month
	let beginning_of_month = Utc.with_ymd_and_hms(current_date.year(), current_date.month(), 1, 0, 0, 0);

	Ok(beginning_of_month.unwrap().timestamp_millis())
}

pub(crate) fn check_id_format(id: &str) -> AppRes<()>
{
	let uuid = Uuid::try_parse(id).map_err(|_e| {
		ServerCoreError::new_msg(
			400,
			ApiErrorCodes::UserNotFound,
			"Id has a wrong format. Make sure to follow the uuid v4 format.",
		)
	})?;

	//uuid v4
	if let Some(Version::Random) = uuid.get_version() {
		return Ok(());
	}

	Err(ServerCoreError::new_msg(
		400,
		ApiErrorCodes::UserNotFound,
		"Id has a wrong format. Make sure to follow the uuid v4 format.",
	))
}

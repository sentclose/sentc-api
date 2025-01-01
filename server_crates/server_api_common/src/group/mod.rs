use rustgram::Request;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;

use crate::group::group_entities::InternalGroupDataComplete;
use crate::ApiErrorCodes;

pub mod group_entities;
pub(crate) mod group_model;

pub const GROUP_TYPE_NORMAL: i32 = 0;
pub const GROUP_TYPE_USER: i32 = 1;

pub use self::group_model::{get_internal_group_data, get_internal_group_user_data, get_user_from_parent_groups};

pub fn get_group_user_data_from_req(req: &Request) -> AppRes<&InternalGroupDataComplete>
{
	match req.extensions().get::<InternalGroupDataComplete>() {
		Some(e) => Ok(e),
		None => {
			Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::GroupAccess,
				"No access to this group",
			))
		},
	}
}

pub mod group_controller;
pub mod group_entities;
mod group_key_rotation;
pub mod group_light_controller;
pub(crate) mod group_model;
pub mod group_service;
mod group_user;

pub(crate) use group_controller::*;
pub(crate) use group_key_rotation::*;
pub(crate) use group_light_controller::*;
pub(crate) use group_user::*;
use rustgram::Request;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;

pub use self::group_key_rotation::group_key_rotation_controller;
pub use self::group_user::{group_user_controller, group_user_service};
use crate::group::group_entities::InternalGroupDataComplete;
use crate::util::api_res::ApiErrorCodes;

pub const GROUP_TYPE_NORMAL: i32 = 0;
pub const GROUP_TYPE_USER: i32 = 1;

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

mod group_controller;
pub(crate) mod group_entities;
mod group_key_rotation;
pub(crate) mod group_model;
mod group_user;

pub(crate) use group_controller::*;
pub(crate) use group_key_rotation::*;
pub(crate) use group_user::*;
use rustgram::Request;

pub use self::group_user::group_user_service;
use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::group::group_entities::InternalGroupDataComplete;

fn get_group_user_data_from_req(req: &Request) -> AppRes<&InternalGroupDataComplete>
{
	match req.extensions().get::<InternalGroupDataComplete>() {
		Some(e) => Ok(e),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::GroupAccess,
				"No access to this group".to_string(),
				None,
			))
		},
	}
}

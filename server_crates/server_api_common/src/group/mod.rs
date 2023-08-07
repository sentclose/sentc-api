use std::future::Future;

use rustgram::Request;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::{GroupId, UserId};

use crate::group::group_entities::{InternalGroupDataComplete, InternalUserGroupData, InternalUserGroupDataFromParent};
use crate::ApiErrorCodes;

pub mod group_entities;
pub(crate) mod group_model;

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

pub fn get_user_from_parent_groups<'a>(
	group_id: impl Into<GroupId> + 'a,
	user_id: impl Into<UserId> + 'a,
) -> impl Future<Output = AppRes<Option<InternalUserGroupDataFromParent>>> + 'a
{
	group_model::get_user_from_parent_groups(group_id, user_id)
}

pub fn get_internal_group_user_data<'a>(
	group_id: impl Into<GroupId> + 'a,
	user_id: impl Into<UserId> + 'a,
) -> impl Future<Output = AppRes<Option<InternalUserGroupData>>> + 'a
{
	group_model::get_internal_group_user_data(group_id, user_id)
}

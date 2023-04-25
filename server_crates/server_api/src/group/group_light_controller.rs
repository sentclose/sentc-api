use std::future::Future;

use rustgram::Request;
use sentc_crypto_common::group::GroupCreateOutput;
use sentc_crypto_common::GroupId;
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::res::{echo, JRes};

use crate::group::{get_group_user_data_from_req, group_service, GROUP_TYPE_NORMAL};
use crate::sentc_app_utils::{check_endpoint_with_app_options, get_app_data_from_req, Endpoint};
use crate::sentc_user_jwt_service::get_jwt_data_from_param;
use crate::util::api_res::ApiErrorCodes;

pub fn create_light(req: Request) -> impl Future<Output = JRes<GroupCreateOutput>>
{
	create_group_light(req, None, None, None, false)
}

pub async fn create_child_group_light(req: Request) -> JRes<GroupCreateOutput>
{
	//this is called in the group mw from the parent group id
	let group_data = get_group_user_data_from_req(&req)?;
	let parent_group_id = Some(group_data.group_data.id.to_string());
	let user_rank = Some(group_data.user_data.rank);

	//a connected group can also got children but these children will be a connected group too
	let is_connected_group = group_data.group_data.is_connected_group;

	create_group_light(req, parent_group_id, user_rank, None, is_connected_group).await
}

pub async fn create_connected_group_from_group_light(req: Request) -> JRes<GroupCreateOutput>
{
	/*
	- A connected group is a group where other groups can join or can get invited, not only users.
	- A connected group can also got children (which are marked as connected group too)
	- A connected group cannot be created from a already connected group.
		Because the users of the one connected group cannot access the connected group.
		So only non connected groups can create connected groups.

	- Users can join both groups
	 */

	//the same as parent group, but this time with the group as member, not as parent
	let group_data = get_group_user_data_from_req(&req)?;
	let connected_group_id = Some(group_data.group_data.id.to_string());
	let user_rank = Some(group_data.user_data.rank);

	if group_data.group_data.is_connected_group {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::GroupConnectedFromConnected,
			"Can't create a connected group from a connected group",
		));
	}

	create_group_light(req, None, user_rank, connected_group_id, true).await
}

async fn create_group_light(
	req: Request,
	parent_group_id: Option<GroupId>,
	user_rank: Option<i32>,
	connected_group: Option<GroupId>,
	is_connected_group: bool,
) -> JRes<GroupCreateOutput>
{
	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::GroupCreate)?;

	let user = get_jwt_data_from_param(&req)?;

	let group_id = group_service::create_group_light(
		&app.app_data.app_id,
		&user.id,
		GROUP_TYPE_NORMAL,
		parent_group_id,
		user_rank,
		connected_group,
		is_connected_group,
	)
	.await?;

	echo(GroupCreateOutput {
		group_id,
	})
}

//__________________________________________________________________________________________________

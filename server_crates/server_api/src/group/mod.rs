pub(crate) mod group_entities;
pub(crate) mod group_model;

use rustgram::Request;
use sentc_crypto_common::group::{CreateData, GroupCreateOutput, GroupDeleteServerOutput, GroupKeysForNewMemberServerInput};
use sentc_crypto_common::server_default::ServerSuccessOutput;

use crate::core::api_res::{echo, echo_success, ApiErrorCodes, AppRes, HttpErr, JRes};
use crate::core::cache;
use crate::core::cache::GROUP_DATA_CACHE;
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::core::url_helper::get_name_param_from_req;
use crate::group::group_entities::InternalGroupDataComplete;
use crate::user::jwt::get_jwt_data_from_param;

pub(crate) async fn create(mut req: Request) -> JRes<GroupCreateOutput>
{
	let body = get_raw_body(&mut req).await?;

	let user = get_jwt_data_from_param(&req)?;

	let input: CreateData = bytes_to_json(&body)?;

	let group_id = group_model::create(user.sub.to_string(), user.id.to_string(), input).await?;

	//delete cache when a wrong cache was created before (for this id)
	let key_group = GROUP_DATA_CACHE.to_string() + group_id.as_str();
	cache::delete(key_group.as_str()).await;

	let out = GroupCreateOutput {
		group_id,
	};

	echo(out)
}

pub(crate) async fn delete(req: Request) -> JRes<GroupDeleteServerOutput>
{
	let group_data = get_group_user_data_from_req(&req)?;

	group_model::delete(
		group_data.group_data.app_id.to_string(),
		group_data.group_data.group_id.to_string(),
		group_data.user_data.rank,
	)
	.await?;

	//don't delete cache for each group user, but for the group
	let key_group = GROUP_DATA_CACHE.to_string() + group_data.group_data.group_id.as_str();
	cache::delete(key_group.as_str()).await;

	let out = GroupDeleteServerOutput {
		group_id: group_data.group_data.group_id.to_string(),
	};

	echo(out)
}

pub async fn invite_request(mut req: Request) -> JRes<ServerSuccessOutput>
{
	//no the accept invite, but the keys are prepared for the invited user
	//don't save this values in the group user keys table, but in the invite table

	let body = get_raw_body(&mut req).await?;

	let invited_user = get_name_param_from_req(&req, "invited_user")?;
	let group_data = get_group_user_data_from_req(&req)?;

	let input: GroupKeysForNewMemberServerInput = bytes_to_json(&body)?;

	group_model::invite_request(
		group_data.group_data.group_id.to_string(),
		group_data.user_data.rank,
		invited_user.to_string(),
		input.0,
	)
	.await?;

	echo_success()
}

//TODO delete cache for user when he joined the group
//TODO delete cache for user when he exit the group
//TODO delete cache for user when change his rank

pub(crate) async fn get(_req: Request) -> Result<String, HttpErr>
{
	Ok(format!("group"))
}

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

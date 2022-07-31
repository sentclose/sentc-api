use rustgram::Request;
use sentc_crypto_common::group::{CreateData, GroupCreateOutput, GroupDeleteServerOutput, GroupKeyServerOutput, GroupServerData};

use crate::core::api_res::{echo, ApiErrorCodes, HttpErr, JRes};
use crate::core::cache;
use crate::core::cache::INTERNAL_GROUP_DATA_CACHE;
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};
use crate::group::{get_group_user_data_from_req, group_model};
use crate::user::jwt::get_jwt_data_from_param;

pub(crate) async fn create(mut req: Request) -> JRes<GroupCreateOutput>
{
	let body = get_raw_body(&mut req).await?;

	let user = get_jwt_data_from_param(&req)?;

	let input: CreateData = bytes_to_json(&body)?;

	let group_id = group_model::create(user.sub.to_string(), user.id.to_string(), input).await?;

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
		group_data.group_data.id.to_string(),
		group_data.user_data.rank,
	)
	.await?;

	//don't delete cache for each group user, but for the group
	let key_group = INTERNAL_GROUP_DATA_CACHE.to_string() + group_data.group_data.app_id.as_str() + "_" + group_data.group_data.id.as_str();
	cache::delete(key_group.as_str()).await;

	let out = GroupDeleteServerOutput {
		group_id: group_data.group_data.id.to_string(),
	};

	echo(out)
}

pub(crate) async fn get_user_group_data(req: Request) -> JRes<GroupServerData>
{
	//don't get the group data from mw cache, this is done by the model fetch

	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	let (user_group_data, user_keys) = group_model::get_user_group_data(user.sub.to_string(), user.id.to_string(), group_id.to_string()).await?;

	let mut keys: Vec<GroupKeyServerOutput> = Vec::with_capacity(user_keys.len());

	for user_key in user_keys {
		keys.push(user_key.into());
	}

	let key_update = group_model::check_for_key_update(user.sub.to_string(), user.sub.to_string(), group_id.to_string()).await?;

	let out = GroupServerData {
		group_id: user_group_data.id,
		parent_group_id: user_group_data.parent_group_id,
		keys,
		key_update,
		rank: user_group_data.rank,
		created_time: user_group_data.created_time,
		joined_time: user_group_data.joined_time,
	};

	echo(out)
}

pub(crate) async fn get_user_group_keys(req: Request) -> JRes<Vec<GroupKeyServerOutput>>
{
	//don't get the group data from mw cache, this is done by the model fetch

	let user = get_jwt_data_from_param(&req)?;
	let req_params = get_params(&req)?;
	let group_id = get_name_param_from_params(req_params, "group_id")?;

	let last_fetched_time = get_name_param_from_params(req_params, "last_fetched_time")?;
	let last_fetched_time: u128 = last_fetched_time.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched time is wrong".to_string(),
			None,
		)
	})?;

	let user_keys = group_model::get_user_group_keys(
		user.sub.to_string(),
		group_id.to_string(),
		user.id.to_string(),
		last_fetched_time,
	)
	.await?;

	let mut keys: Vec<GroupKeyServerOutput> = Vec::with_capacity(user_keys.len());
	for user_key in user_keys {
		keys.push(user_key.into());
	}

	echo(keys)
}

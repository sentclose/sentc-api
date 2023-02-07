use std::future::Future;

use rustgram::Request;
use sentc_crypto_common::content::{ContentCreateOutput, CreateData};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use server_core::input_helper::{bytes_to_json, get_raw_body};
use server_core::res::{echo, JRes};
use server_core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params, get_time_from_url_param};

use crate::content_management::content_entity::{ContentItemAccess, ListContentItem};
use crate::content_management::content_service::ContentRelatedType;
use crate::content_management::{content_model, content_service};
use crate::customer_app::app_util::{check_endpoint_with_app_options, get_app_data_from_req, Endpoint};
use crate::group::get_group_user_data_from_req;
use crate::user::jwt::get_jwt_data_from_param;
use crate::util::api_res::echo_success;

/**
## Category

- Create category
- update name
- delete

## Content
- create and select 0 - n cat.
- update: update the item
- delete the item

### Access
- get all new items for user (from all groups and sub groups incl. group as member and child groups)
- for group as member only the direct connected groups
	because the user can't access a connected group which is also connected to another group

- get all items of a cat
- get all items of a group
- get all items of a group from a cat

- the same fetch with last item
 */

pub(crate) fn create_non_related_content(req: Request) -> impl Future<Output = JRes<ContentCreateOutput>>
{
	create_content(req, ContentRelatedType::None)
}

pub(crate) fn create_user_content(req: Request) -> impl Future<Output = JRes<ContentCreateOutput>>
{
	create_content(req, ContentRelatedType::User)
}

pub(crate) fn create_group_content(req: Request) -> impl Future<Output = JRes<ContentCreateOutput>>
{
	create_content(req, ContentRelatedType::Group)
}

async fn create_content(mut req: Request, content_related_type: ContentRelatedType) -> JRes<ContentCreateOutput>
{
	let body = get_raw_body(&mut req).await?;

	let app = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	check_endpoint_with_app_options(app, Endpoint::ForceServer)?;

	let (group_id, user_id) = match content_related_type {
		ContentRelatedType::None => (None, None),
		ContentRelatedType::User => {
			//get the user id from the url param
			let user_id = get_name_param_from_req(&req, "user_id")?;

			(None, Some(user_id))
		},
		ContentRelatedType::Group => {
			let group_data = get_group_user_data_from_req(&req)?;

			(Some(group_data.group_data.id.as_str()), None)
		},
	};

	let input: CreateData = bytes_to_json(&body)?;

	//no rank check for group because the req is made from the customer server.
	// so this server must handle the access

	let content_id = content_service::create_content(&app.app_data.app_id, &user.id, input, group_id, user_id).await?;

	let out = ContentCreateOutput {
		content_id,
	};

	echo(out)
}

pub(crate) async fn delete_content_by_id(req: Request) -> JRes<ServerSuccessOutput>
{
	//again no rank checks for groups because this is sent form the own backend

	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::ForceServer)?;

	let id = get_name_param_from_req(&req, "content_id")?;

	content_service::delete_content_by_id(&app.app_data.app_id, id).await?;

	echo_success()
}

pub(crate) async fn delete_content_by_item(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::ForceServer)?;

	let item = get_name_param_from_req(&req, "item")?;

	content_service::delete_content_by_item(&app.app_data.app_id, item).await?;

	echo_success()
}

pub(crate) async fn check_access_to_content_by_item(req: Request) -> JRes<ContentItemAccess>
{
	let app = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	//TODO endpoint check not force server
	check_endpoint_with_app_options(app, Endpoint::ForceServer)?;

	let item = get_name_param_from_req(&req, "item")?;

	let out = content_model::check_access_to_content_by_item(&app.app_data.app_id, &user.id, item).await?;

	echo(out)
}

pub(crate) fn get_content_all(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::None, false)
}

pub(crate) fn get_content_for_user(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::User, false)
}

pub(crate) fn get_content_for_group(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::Group, false)
}

pub(crate) fn get_content_all_from_cat(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::None, true)
}

pub(crate) fn get_content_for_user_from_cat(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::User, true)
}

pub(crate) fn get_content_for_group_from_cat(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::Group, true)
}

async fn get_content(req: Request, content_related_type: ContentRelatedType, cat: bool) -> JRes<Vec<ListContentItem>>
{
	let app = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	//TODO endpoint check not force server
	check_endpoint_with_app_options(app, Endpoint::ForceServer)?;

	let params = get_params(&req)?;
	let last_id = get_name_param_from_params(params, "last_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let cat_id = match cat {
		false => None,
		true => Some(get_name_param_from_params(params, "cat_id")?),
	};

	let list = match content_related_type {
		ContentRelatedType::None => content_model::get_content(&app.app_data.app_id, &user.id, last_fetched_time, last_id, cat_id).await?,
		ContentRelatedType::Group => {
			let group_data = get_group_user_data_from_req(&req)?;

			content_model::get_content_for_group(
				&app.app_data.app_id,
				&group_data.group_data.id,
				last_fetched_time,
				last_id,
				cat_id,
			)
			.await?
		},
		ContentRelatedType::User => content_model::get_content_to_user(&app.app_data.app_id, &user.id, last_fetched_time, last_id, cat_id).await?,
	};

	echo(list)
}

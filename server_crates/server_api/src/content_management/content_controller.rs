use std::future::Future;

use rustgram::Request;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, JRes, ServerSuccessOutput};
use rustgram_server_util::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params, get_time_from_url_param};
use sentc_crypto_common::content::{ContentCreateOutput, CreateData};
use server_api_common::customer_app::{check_endpoint_with_app_options, get_app_data_from_req, Endpoint};
use server_api_common::group::get_group_user_data_from_req;
use server_api_common::user::get_jwt_data_from_param;

use crate::content_management::content_entity::{ContentItemAccess, ListContentItem};
use crate::content_management::content_service::ContentRelatedType;
use crate::content_management::{content_model, content_service};
use crate::util::api_res::ApiErrorCodes;

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

			(None, Some(user_id.to_string()))
		},
		ContentRelatedType::Group => {
			let group_data = get_group_user_data_from_req(&req)?;

			(Some(group_data.group_data.id.clone()), None)
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

	check_endpoint_with_app_options(app, Endpoint::Content)?;

	let item = get_name_param_from_req(&req, "item")?;

	let out = content_model::check_access_to_content_by_item(&app.app_data.app_id, &user.id, item).await?;

	echo(out)
}

//==================================================================================================

const LIMIT_FETCH_SMALL: &str = "20";
const LIMIT_FETCH_MED: &str = "50";
const LIMIT_FETCH_LARGE: &str = "70";
const LIMIT_FETCH_X_LARGE: &str = "100";

pub(crate) fn get_content_all_small(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::None, false, LIMIT_FETCH_SMALL)
}

pub(crate) fn get_content_all_med(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::None, false, LIMIT_FETCH_MED)
}

pub(crate) fn get_content_all_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::None, false, LIMIT_FETCH_LARGE)
}

pub(crate) fn get_content_all_x_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::None, false, LIMIT_FETCH_X_LARGE)
}

//__________________________________________________________________________________________________

pub(crate) fn get_content_for_user_small(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::User, false, LIMIT_FETCH_SMALL)
}

pub(crate) fn get_content_for_user_med(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::User, false, LIMIT_FETCH_MED)
}

pub(crate) fn get_content_for_user_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::User, false, LIMIT_FETCH_LARGE)
}

pub(crate) fn get_content_for_user_x_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::User, false, LIMIT_FETCH_X_LARGE)
}

//__________________________________________________________________________________________________

pub(crate) fn get_content_for_group_small(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::Group, false, LIMIT_FETCH_SMALL)
}

pub(crate) fn get_content_for_group_med(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::Group, false, LIMIT_FETCH_MED)
}

pub(crate) fn get_content_for_group_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::Group, false, LIMIT_FETCH_LARGE)
}

pub(crate) fn get_content_for_group_x_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::Group, false, LIMIT_FETCH_X_LARGE)
}

//__________________________________________________________________________________________________

pub(crate) fn get_content_all_from_cat_small(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::None, true, LIMIT_FETCH_SMALL)
}

pub(crate) fn get_content_all_from_cat_med(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::None, true, LIMIT_FETCH_MED)
}

pub(crate) fn get_content_all_from_cat_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::None, true, LIMIT_FETCH_LARGE)
}

pub(crate) fn get_content_all_from_cat_x_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::None, true, LIMIT_FETCH_X_LARGE)
}

//__________________________________________________________________________________________________

pub(crate) fn get_content_for_user_from_cat_small(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::User, true, LIMIT_FETCH_SMALL)
}

pub(crate) fn get_content_for_user_from_cat_med(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::User, true, LIMIT_FETCH_MED)
}

pub(crate) fn get_content_for_user_from_cat_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::User, true, LIMIT_FETCH_LARGE)
}

pub(crate) fn get_content_for_user_from_cat_x_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::User, true, LIMIT_FETCH_X_LARGE)
}

//__________________________________________________________________________________________________

pub(crate) fn get_content_for_group_from_cat_small(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::Group, true, LIMIT_FETCH_SMALL)
}

pub(crate) fn get_content_for_group_from_cat_med(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::Group, true, LIMIT_FETCH_MED)
}

pub(crate) fn get_content_for_group_from_cat_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::Group, true, LIMIT_FETCH_LARGE)
}

pub(crate) fn get_content_for_group_from_cat_x_large(req: Request) -> impl Future<Output = JRes<Vec<ListContentItem>>>
{
	get_content(req, ContentRelatedType::Group, true, LIMIT_FETCH_X_LARGE)
}

//__________________________________________________________________________________________________

async fn get_content(req: Request, content_related_type: ContentRelatedType, cat: bool, limit: &str) -> JRes<Vec<ListContentItem>>
{
	let app = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	match limit {
		LIMIT_FETCH_SMALL => check_endpoint_with_app_options(app, Endpoint::ContentSmall)?,
		LIMIT_FETCH_MED => check_endpoint_with_app_options(app, Endpoint::ContentMed)?,
		LIMIT_FETCH_LARGE => check_endpoint_with_app_options(app, Endpoint::ContentLarge)?,
		LIMIT_FETCH_X_LARGE => check_endpoint_with_app_options(app, Endpoint::ContentXLarge)?,
		_ => {
			return Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::AppAction,
				"No valid limit",
			))
		},
	}

	let params = get_params(&req)?;
	let last_id = get_name_param_from_params(params, "last_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let cat_id = match cat {
		false => None,
		true => Some(get_name_param_from_params(params, "cat_id")?.to_string()),
	};

	let list = match content_related_type {
		ContentRelatedType::None => {
			content_model::get_content(
				&app.app_data.app_id,
				&user.id,
				last_fetched_time,
				last_id,
				cat_id,
				limit,
			)
			.await?
		},
		ContentRelatedType::Group => {
			let group_data = get_group_user_data_from_req(&req)?;

			content_model::get_content_for_group(
				&app.app_data.app_id,
				&group_data.group_data.id,
				last_fetched_time,
				last_id,
				cat_id,
				limit,
			)
			.await?
		},
		ContentRelatedType::User => {
			content_model::get_content_to_user(
				&app.app_data.app_id,
				&user.id,
				last_fetched_time,
				last_id,
				cat_id,
				limit,
			)
			.await?
		},
	};

	echo(list)
}

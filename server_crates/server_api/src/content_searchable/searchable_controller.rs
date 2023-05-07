use std::future::Future;

use rustgram::Request;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, JRes, ServerSuccessOutput};
use rustgram_server_util::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params, get_query_params, get_time_from_url_param};
use sentc_crypto_common::content_searchable::SearchCreateData;

use crate::content_searchable::searchable_entities::ListSearchItem;
use crate::content_searchable::searchable_service;
use crate::get_group_user_data_from_req;
use crate::sentc_app_utils::{check_endpoint_with_app_options, get_app_data_from_req, Endpoint};
use crate::util::api_res::ApiErrorCodes;

pub(crate) async fn create(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::ForceServer)?;

	let app_id = &app.app_data.app_id;

	let group_data = get_group_user_data_from_req(&req)?;
	let group_id = Some(group_data.group_data.id.clone());

	let input: SearchCreateData = bytes_to_json(&body)?;

	searchable_service::create_searchable_content(app_id, input, group_id, None).await?;

	echo_success()
}

pub(crate) async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::ForceServer)?;

	let item = get_name_param_from_req(&req, "item_ref")?;

	searchable_service::delete_item(&app.app_data.app_id, item).await?;

	echo_success()
}

pub(crate) async fn delete_by_cat(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::ForceServer)?;

	let item = get_name_param_from_req(&req, "item_ref")?;
	let cat_id = get_name_param_from_req(&req, "cat_id")?;

	searchable_service::delete_item_by_cat(&app.app_data.app_id, item, cat_id).await?;

	echo_success()
}

pub(crate) fn search_all(req: Request) -> impl Future<Output = JRes<Vec<ListSearchItem>>>
{
	search(req, false)
}

pub(crate) fn search_cat(req: Request) -> impl Future<Output = JRes<Vec<ListSearchItem>>>
{
	search(req, true)
}

async fn search(req: Request, cat: bool) -> JRes<Vec<ListSearchItem>>
{
	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::ContentSearch)?;

	let params = get_params(&req)?;
	let last_id = get_name_param_from_params(params, "last_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let url_query = get_query_params(&req)?;
	let search_hash = match url_query.get("search") {
		Some(q) => q,
		None => {
			return Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::ContentSearchableQueryMissing,
				"The search query is missing",
			));
		},
	};

	let group_data = get_group_user_data_from_req(&req)?;

	let cat_id = match cat {
		false => None,
		true => Some(get_name_param_from_params(params, "cat_id")?.to_string()),
	};

	let list = searchable_service::search_item_for_group(
		&app.app_data.app_id,
		&group_data.group_data.id,
		search_hash,
		last_fetched_time,
		last_id,
		50,
		cat_id,
	)
	.await?;

	echo(list)
}

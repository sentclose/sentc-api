use sentc_crypto_common::content_searchable::SearchCreateData;
use sentc_crypto_common::{AppId, CategoryId, ContentId, GroupId, UserId};

use crate::content_searchable::searchable_entities::ListSearchItem;
use crate::content_searchable::searchable_model;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

pub async fn create_searchable_content(app_id: AppId, data: SearchCreateData, group_id: Option<GroupId>, user_id: Option<UserId>) -> AppRes<()>
{
	if data.item_ref.is_empty() {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentSearchableItemRefNotSet,
			"Item is not set".to_string(),
			None,
		));
	}

	if data.item_ref.len() > 50 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentSearchableItemRefTooBig,
			"Item ref is too big. Only 50 characters are allowed".to_string(),
			None,
		));
	}

	if data.hashes.is_empty() {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentSearchableNoHashes,
			"No hashes sent".to_string(),
			None,
		));
	}

	if data.hashes.len() > 200 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentSearchableTooManyHashes,
			"Item is too big. Only 200 characters are allowed".to_string(),
			None,
		));
	}

	searchable_model::create(app_id, data, group_id, user_id).await
}

pub async fn delete_item(app_id: AppId, item_ref: String) -> AppRes<()>
{
	if item_ref.is_empty() {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentSearchableItemRefNotSet,
			"Item is not set".to_string(),
			None,
		));
	}

	if item_ref.len() > 50 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentSearchableItemRefTooBig,
			"Item ref is too big. Only 50 characters are allowed".to_string(),
			None,
		));
	}

	searchable_model::delete(app_id, item_ref).await
}

pub async fn delete_item_by_cat(app_id: AppId, item_ref: String, cat: String) -> AppRes<()>
{
	if item_ref.is_empty() {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentSearchableItemRefNotSet,
			"Item is not set".to_string(),
			None,
		));
	}

	if item_ref.len() > 50 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentSearchableItemRefTooBig,
			"Item ref is too big. Only 50 characters are allowed".to_string(),
			None,
		));
	}

	searchable_model::delete_by_cat(app_id, item_ref, cat).await
}

pub async fn search_item_for_group(
	app_id: AppId,
	group_id: GroupId,
	search_hash: String,
	last_fetched_time: u128,
	last_id: ContentId,
	limit: u32,
	cat_id: Option<CategoryId>,
) -> AppRes<Vec<ListSearchItem>>
{
	if search_hash.is_empty() {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentSearchableNoHashes,
			"No hash sent".to_string(),
			None,
		));
	}

	if search_hash.len() > 200 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentSearchableTooManyHashes,
			"Hash is too big.".to_string(),
			None,
		));
	}

	let limit = if limit > 100 { 100 } else { limit };

	searchable_model::search_item_for_group(
		app_id,
		group_id,
		search_hash,
		last_fetched_time,
		last_id,
		limit,
		cat_id,
	)
	.await
}

use sentc_crypto_common::content_searchable::SearchCreateData;
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::res::AppRes;

use crate::content_searchable::searchable_entities::ListSearchItem;
use crate::content_searchable::searchable_model;
use crate::util::api_res::ApiErrorCodes;

pub async fn create_searchable_content(app_id: &str, data: SearchCreateData, group_id: Option<&str>, user_id: Option<&str>) -> AppRes<()>
{
	if data.item_ref.is_empty() {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentSearchableItemRefNotSet,
			"Item is not set",
		));
	}

	if data.item_ref.len() > 50 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentSearchableItemRefTooBig,
			"Item ref is too big. Only 50 characters are allowed",
		));
	}

	if data.hashes.is_empty() {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentSearchableNoHashes,
			"No hashes sent",
		));
	}

	if data.hashes.len() > 200 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentSearchableTooManyHashes,
			"Item is too big. Only 200 characters are allowed",
		));
	}

	searchable_model::create(app_id, data, group_id, user_id).await
}

pub async fn delete_item(app_id: &str, item_ref: &str) -> AppRes<()>
{
	if item_ref.is_empty() {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentSearchableItemRefNotSet,
			"Item is not set",
		));
	}

	if item_ref.len() > 50 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentSearchableItemRefTooBig,
			"Item ref is too big. Only 50 characters are allowed",
		));
	}

	searchable_model::delete(app_id, item_ref).await
}

pub async fn delete_item_by_cat(app_id: &str, item_ref: &str, cat: &str) -> AppRes<()>
{
	if item_ref.is_empty() {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentSearchableItemRefNotSet,
			"Item is not set",
		));
	}

	if item_ref.len() > 50 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentSearchableItemRefTooBig,
			"Item ref is too big. Only 50 characters are allowed",
		));
	}

	searchable_model::delete_by_cat(app_id, item_ref, cat).await
}

pub async fn search_item_for_group(
	app_id: &str,
	group_id: &str,
	search_hash: &str,
	last_fetched_time: u128,
	last_id: &str,
	limit: u32,
	cat_id: Option<&str>,
) -> AppRes<Vec<ListSearchItem>>
{
	if search_hash.is_empty() {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentSearchableNoHashes,
			"No hash sent",
		));
	}

	if search_hash.len() > 200 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentSearchableTooManyHashes,
			"Hash is too big.",
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

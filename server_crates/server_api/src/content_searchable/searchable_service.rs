use sentc_crypto_common::content_searchable::SearchCreateData;
use sentc_crypto_common::{AppId, CategoryId, ContentId, GroupId, UserId};
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::res::AppRes;

use crate::content_searchable::searchable_entities::ListSearchItem;
use crate::content_searchable::searchable_model;
use crate::util::api_res::ApiErrorCodes;

pub async fn create_searchable_content(
	app_id: impl Into<AppId>,
	data: SearchCreateData,
	group_id: Option<GroupId>,
	user_id: Option<UserId>,
) -> AppRes<()>
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

pub async fn delete_item(app_id: impl Into<AppId>, item_ref: impl Into<String>) -> AppRes<()>
{
	let item_ref = item_ref.into();

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

pub async fn delete_item_by_cat(app_id: impl Into<AppId>, item_ref: impl Into<String>, cat: impl Into<CategoryId>) -> AppRes<()>
{
	let item_ref = item_ref.into();

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
	app_id: impl Into<AppId>,
	group_id: impl Into<GroupId>,
	search_hash: impl Into<String>,
	last_fetched_time: u128,
	last_id: impl Into<ContentId>,
	limit: u32,
	cat_id: Option<CategoryId>,
) -> AppRes<Vec<ListSearchItem>>
{
	let search_hash = search_hash.into();

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

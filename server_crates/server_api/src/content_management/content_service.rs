use std::future::Future;

use sentc_crypto_common::content::CreateData;
use sentc_crypto_common::{AppId, ContentId, GroupId, UserId};
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::res::AppRes;

use crate::content_management::content_model_edit;
use crate::util::api_res::ApiErrorCodes;

pub enum ContentRelatedType
{
	Group,
	User,
	None,
}

pub async fn create_content(
	app_id: impl Into<AppId>,
	creator_id: impl Into<UserId>,
	data: CreateData,
	group_id: Option<GroupId>,
	user_id: Option<UserId>,
) -> AppRes<ContentId>
{
	if data.item.is_empty() {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentItemNotSet,
			"Item is not set",
		));
	}

	if data.item.len() > 50 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentItemTooBig,
			"Item is too big. Only 50 characters are allowed",
		));
	}

	content_model_edit::create_content(app_id, creator_id, data, group_id, user_id).await
}

pub fn delete_content_by_id<'a>(app_id: impl Into<AppId> + 'a, content_id: impl Into<ContentId> + 'a) -> impl Future<Output = AppRes<()>> + 'a
{
	content_model_edit::delete_content_by_id(app_id, content_id)
}

pub async fn delete_content_by_item(app_id: impl Into<AppId>, item: impl Into<String>) -> AppRes<()>
{
	let item = item.into();

	if item.is_empty() {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentItemNotSet,
			"Item is not set",
		));
	}

	if item.len() > 50 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::ContentItemTooBig,
			"Item is too big. Only 50 characters are allowed",
		));
	}

	content_model_edit::delete_content_by_item(app_id, item).await
}

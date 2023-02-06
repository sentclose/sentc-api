use std::future::Future;

use sentc_crypto_common::content::CreateData;
use sentc_crypto_common::ContentId;
use server_core::str_t;

use crate::content_management::content_model_edit;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

pub enum ContentRelatedType
{
	Group,
	User,
	None,
}

pub async fn create_content(app_id: &str, creator_id: &str, data: CreateData, group_id: Option<&str>, user_id: Option<&str>) -> AppRes<ContentId>
{
	if data.item.is_empty() {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentItemNotSet,
			"Item is not set".to_string(),
			None,
		));
	}

	if data.item.len() > 50 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentItemTooBig,
			"Item is too big. Only 50 characters are allowed".to_string(),
			None,
		));
	}

	content_model_edit::create_content(app_id, creator_id, data, group_id, user_id).await
}

pub fn delete_content_by_id<'a>(app_id: str_t!('a), content_id: str_t!('a)) -> impl Future<Output = AppRes<()>> + 'a
{
	content_model_edit::delete_content_by_id(app_id, content_id)
}

pub async fn delete_content_by_item(app_id: &str, item: &str) -> AppRes<()>
{
	if item.is_empty() {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentItemNotSet,
			"Item is not set".to_string(),
			None,
		));
	}

	if item.len() > 50 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentItemTooBig,
			"Item is too big. Only 50 characters are allowed".to_string(),
			None,
		));
	}

	content_model_edit::delete_content_by_item(app_id, item).await
}

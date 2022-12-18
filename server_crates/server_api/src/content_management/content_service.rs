use sentc_crypto_common::content::CreateData;
use sentc_crypto_common::{AppId, ContentId, GroupId, UserId};

use crate::content_management::content_model;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

pub enum ContentRelatedType
{
	Group,
	User,
	None,
}

pub async fn create_content(
	app_id: AppId,
	creator_id: UserId,
	data: CreateData,
	group_id: Option<GroupId>,
	user_id: Option<UserId>,
) -> AppRes<ContentId>
{
	if data.item.is_empty() {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentCreateItemNotSet,
			"Item is not set".to_string(),
			None,
		));
	}

	if data.item.len() > 50 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentCreateItemTooBig,
			"Item is too big. Only 50 characters are allowed".to_string(),
			None,
		));
	}

	if data.cat_ids.len() > 50 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::ContentCreateItemTooManyCat,
			"Too many categories chose for this item. Max is 50.".to_string(),
			None,
		));
	}

	content_model::create_content(app_id, creator_id, data, group_id, user_id).await
}

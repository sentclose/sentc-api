use sentc_crypto_common::user::UserIdentifierAvailableServerOutput;

use crate::core::api_err::{ApiErrorCodes, HttpErr};
use crate::user::user_entities::UserEntity;
use crate::user::user_model;

pub(crate) async fn exists() -> Result<String, HttpErr>
{
	let user_id = "058ed2e6-3880-4a7c-ab3b-fd2f5755ea43"; //get this from the url param

	let exists = user_model::check_user_exists(user_id).await?;

	UserIdentifierAvailableServerOutput {
		user_identifier: user_id.to_string(),
		available: exists,
	}
	.to_string()
	.map_err(|e| {
		HttpErr::new(
			400,
			ApiErrorCodes::JsonToString,
			"Json to string failed",
			Some(format!("err json to string: {:?}", e)),
		)
	})
}

pub(crate) async fn get() -> Result<UserEntity, HttpErr>
{
	let user_id = "abc"; //get this from the url param

	//
	let user = user_model::get_user(user_id).await?;

	Ok(user)
}

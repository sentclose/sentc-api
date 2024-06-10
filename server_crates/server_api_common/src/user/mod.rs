use std::future::Future;

use rustgram::Request;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::{AppId, UserId};

use crate::user::user_entity::UserJwtEntity;
use crate::ApiErrorCodes;

pub mod captcha;
pub mod jwt;
pub mod user_entity;
pub(crate) mod user_model;

pub fn get_jwt_data_from_param(req: &Request) -> Result<&UserJwtEntity, ServerCoreError>
{
	//p should always be some for non-optional jwt
	match req.extensions().get::<Option<UserJwtEntity>>() {
		Some(Some(p)) => Ok(p),
		_ => {
			Err(ServerCoreError::new_msg(
				400,
				ApiErrorCodes::JwtNotFound,
				"No valid jwt",
			))
		},
	}
}

pub fn check_user_in_app_by_user_id<'a>(app_id: impl Into<AppId> + 'a, user_id: impl Into<UserId> + 'a) -> impl Future<Output = AppRes<bool>> + 'a
{
	user_model::check_user_in_app(app_id, user_id)
}

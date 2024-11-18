use rustgram::Request;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};

use crate::user::user_entity::UserJwtEntity;
use crate::ApiErrorCodes;

pub mod jwt;
pub mod user_entity;
pub(crate) mod user_model;

pub use user_model::check_user_in_app as check_user_in_app_by_user_id;

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

/**
# Generated route files by rustgram route builder.

Please do not modify this file. Any changes will be overridden by the next route build.
Use the returned router instead
 */
use rustgram::{r, Router};

pub(crate) fn routes(router: &mut Router)
{
	router.post(
		"/api/v1/file",
		r(crate::file_controller::register_file)
			.add(server_api_common::middleware::jwt::jwt_transform)
			.add(server_api_common::middleware::app_token::app_token_transform),
	);
	router.put(
		"/api/v1/file/:file_id",
		r(crate::file_controller::update_file_name)
			.add(server_api_common::middleware::jwt::jwt_transform)
			.add(server_api_common::middleware::app_token::app_token_transform),
	);
	router.delete(
		"/api/v1/file/:file_id",
		r(crate::file_controller::delete_file)
			.add(server_api_common::middleware::jwt::jwt_transform)
			.add(server_api_common::middleware::app_token::app_token_transform),
	);
	router.get(
		"/api/v1/file/:file_id",
		r(crate::file_controller::get_file)
			.add(server_api_common::middleware::jwt::jwt_optional_transform)
			.add(server_api_common::middleware::app_token::app_token_transform),
	);
	router.get(
		"/api/v1/file/:file_id/part_fetch/:last_sequence",
		r(crate::file_controller::get_parts).add(server_api_common::middleware::app_token::app_token_transform),
	);
	router.get(
		"/api/v1/file/part/:part_id",
		r(crate::file_controller::download_part).add(server_api_common::middleware::app_token::app_token_transform),
	);
	router.delete(
		"/api/v1/file/part/:part_id",
		r(crate::file_controller::delete_registered_file_part).add(server_api_common::middleware::app_token::app_token_transform),
	);
	router.post(
		"/api/v1/file/part/:session_id/:seq/:end",
		r(crate::file_controller::upload_part)
			.add(server_api_common::middleware::jwt::jwt_expire_transform)
			.add(server_api_common::middleware::app_token::app_token_transform),
	);
	router.patch(
		"/api/v1/file/part/:session_id/:seq/:end/:user_id",
		r(crate::file_controller::register_file_part).add(server_api_common::middleware::app_token::app_token_transform),
	);
	router.post(
		"/api/v1/group/:group_id/file",
		r(crate::file_controller::register_file_in_group)
			.add(server_api_common::middleware::group::group_transform)
			.add(server_api_common::middleware::jwt::jwt_transform)
			.add(server_api_common::middleware::app_token::app_token_transform),
	);
	router.get(
		"/api/v1/group/:group_id/file/:file_id",
		r(crate::file_controller::get_file_in_group)
			.add(server_api_common::middleware::group::group_transform)
			.add(server_api_common::middleware::jwt::jwt_transform)
			.add(server_api_common::middleware::app_token::app_token_transform),
	);
	router.delete(
		"/api/v1/group/:group_id/file/:file_id",
		r(crate::file_controller::delete_file_in_group)
			.add(server_api_common::middleware::group::group_transform)
			.add(server_api_common::middleware::jwt::jwt_transform)
			.add(server_api_common::middleware::app_token::app_token_transform),
	);
}

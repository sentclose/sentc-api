/**
# Generated route files by rustgram route builder.

Please do not modify this file. Any changes will be overridden by the next route build.
Use the returned router instead
 */
use rustgram::{r, Router};

use crate::middleware::*;

pub(crate) fn routes(router: &mut Router)
{
	router.post(
		"/api/v1/customer/register",
		r(crate::customer::register).add(app_token::app_token_base_app_transform),
	);
	router.post(
		"/api/v1/customer/prepare_login",
		r(crate::customer::prepare_login).add(app_token::app_token_base_app_transform),
	);
	router.post(
		"/api/v1/customer/done_login",
		r(crate::customer::done_login).add(app_token::app_token_base_app_transform),
	);
	router.get(
		"/api/v1/customer/captcha",
		r(crate::customer::customer_captcha).add(app_token::app_token_base_app_transform),
	);
	router.put(
		"/api/v1/customer/password_reset",
		r(crate::customer::prepare_reset_password).add(app_token::app_token_base_app_transform),
	);
	router.put(
		"/api/v1/customer/password_reset_validation",
		r(crate::customer::done_reset_password).add(app_token::app_token_base_app_transform),
	);
	router.put(
		"/api/v1/customer/refresh",
		r(crate::customer::refresh_jwt)
			.add(jwt::jwt_expire_transform)
			.add(app_token::app_token_base_app_transform),
	);
	router.post(
		"/api/v1/customer/register_validation",
		r(crate::customer::done_register).add(jwt::jwt_app_check_transform),
	);
	router.patch(
		"/api/v1/customer/email_resend",
		r(crate::customer::resend_email).add(jwt::jwt_app_check_transform),
	);
	router.put(
		"/api/v1/customer",
		r(crate::customer::update).add(jwt::jwt_app_check_transform),
	);
	router.put(
		"/api/v1/customer/data",
		r(crate::customer::update_data).add(jwt::jwt_app_check_transform),
	);
	router.put(
		"/api/v1/customer/password",
		r(crate::customer::change_password).add(jwt::jwt_app_check_transform),
	);
	router.delete(
		"/api/v1/customer",
		r(crate::customer::delete).add(jwt::jwt_app_check_transform),
	);
	router.get(
		"/api/v1/customer/apps/:last_fetched_time/:last_app_id",
		r(crate::customer_app::get_all_apps).add(jwt::jwt_app_check_transform),
	);
	router.post(
		"/api/v1/customer/app",
		r(crate::customer_app::create_app).add(jwt::jwt_app_check_transform),
	);
	router.get(
		"/api/v1/customer/app/:app_id",
		r(crate::customer_app::get_app_details).add(jwt::jwt_app_check_transform),
	);
	router.put(
		"/api/v1/customer/app/:app_id",
		r(crate::customer_app::update).add(jwt::jwt_app_check_transform),
	);
	router.put(
		"/api/v1/customer/app/:app_id/options",
		r(crate::customer_app::update_options).add(jwt::jwt_app_check_transform),
	);
	router.put(
		"/api/v1/customer/app/:app_id/file_options",
		r(crate::customer_app::update_file_options).add(jwt::jwt_app_check_transform),
	);
	router.delete(
		"/api/v1/customer/app/:app_id",
		r(crate::customer_app::delete).add(jwt::jwt_app_check_transform),
	);
	router.patch(
		"/api/v1/customer/app/:app_id/token_renew",
		r(crate::customer_app::renew_tokens).add(jwt::jwt_app_check_transform),
	);
	router.patch(
		"/api/v1/customer/app/:app_id/new_jwt_keys",
		r(crate::customer_app::add_jwt_keys).add(jwt::jwt_app_check_transform),
	);
	router.get(
		"/api/v1/customer/app/:app_id/jwt",
		r(crate::customer_app::get_jwt_details).add(jwt::jwt_app_check_transform),
	);
	router.delete(
		"/api/v1/customer/app/:app_id/jwt/:jwt_id",
		r(crate::customer_app::delete_jwt_keys).add(jwt::jwt_app_check_transform),
	);
	router.get(
		"/api/v1/user/:user_id",
		r(crate::user::get).add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/user/:user_id/public_key",
		r(crate::user::get_public_key_data).add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/user/:user_id/public_key/:key_id",
		r(crate::user::get_public_key_by_id).add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/user/:user_id/verify_key",
		r(crate::user::get_verify_key_data).add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/user/:user_id/verify_key/:key_id",
		r(crate::user::get_verify_key_by_id).add(app_token::app_token_transform),
	);
	router.post(
		"/api/v1/exists",
		r(crate::user::exists).add(app_token::app_token_transform),
	);
	router.post(
		"/api/v1/register",
		r(crate::user::register).add(app_token::app_token_transform),
	);
	router.post(
		"/api/v1/prepare_login",
		r(crate::user::prepare_login).add(app_token::app_token_transform),
	);
	router.post(
		"/api/v1/done_login",
		r(crate::user::done_login).add(app_token::app_token_transform),
	);
	router.post(
		"/api/v1/user/prepare_register_device",
		r(crate::user::prepare_register_device).add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/refresh",
		r(crate::user::refresh_jwt)
			.add(jwt::jwt_expire_transform)
			.add(app_token::app_token_transform),
	);
	router.post(
		"/api/v1/init",
		r(crate::user::init_user)
			.add(jwt::jwt_expire_transform)
			.add(app_token::app_token_transform),
	);
	router.post(
		"/api/v1/keys/sym_key",
		r(crate::key_management::register_sym_key)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.delete(
		"/api/v1/keys/sym_key/:key_id",
		r(crate::key_management::delete_sym_key)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/keys/sym_key/master_key/:master_key_id/:last_fetched_time/:last_key_id",
		r(crate::key_management::get_all_sym_keys_to_master_key).add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/keys/sym_key/:key_id",
		r(crate::key_management::get_sym_key_by_id).add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/user/device/:last_fetched_time/:last_id",
		r(crate::user::get_devices)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/user",
		r(crate::user::update)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/user/done_register_device",
		r(crate::user::done_register_device)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/user/update_pw",
		r(crate::user::change_password)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/user/reset_pw",
		r(crate::user::reset_password)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.delete(
		"/api/v1/user",
		r(crate::user::delete)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.delete(
		"/api/v1/user/device/:device_id",
		r(crate::user::delete_device)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.post(
		"/api/v1/user/user_keys/rotation",
		r(crate::user::user_group_key_rotation)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/user/user_keys/rotation",
		r(crate::user::get_user_group_keys_for_update)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/user/user_keys/rotation/:key_id",
		r(crate::user::done_key_rotation_for_device)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/user/user_keys/key/:key_id",
		r(crate::user::get_user_key)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/user/user_keys/keys/:last_fetched_time/:last_k_id",
		r(crate::user::get_user_keys)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/user/user_keys/session/:key_session_id",
		r(crate::user::device_key_upload)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.post(
		"/api/v1/group",
		r(crate::group::create)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/group/all/:last_fetched_time/:last_group_id",
		r(crate::group::get_all_groups_for_user)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/group/invite/:last_fetched_time/:last_group_id",
		r(crate::group::get_invite_req)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.patch(
		"/api/v1/group/:group_id/invite",
		r(crate::group::accept_invite)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.delete(
		"/api/v1/group/:group_id/invite",
		r(crate::group::reject_invite)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.patch(
		"/api/v1/group/:group_id/join_req",
		r(crate::group::join_req)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/group/:group_id",
		r(crate::group::get_user_group_data)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/group/:group_id/update_check",
		r(crate::group::get_key_update_for_user)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/group/:group_id/keys/:last_fetched_time/:last_k_id",
		r(crate::group::get_user_group_keys)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/group/:group_id/key/:key_id",
		r(crate::group::get_user_group_key)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/group/:group_id/member/:last_fetched_time/:last_user_id",
		r(crate::group::get_group_member)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.post(
		"/api/v1/group/:group_id/child",
		r(crate::group::create_child_group)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.delete(
		"/api/v1/group/:group_id",
		r(crate::group::delete)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.delete(
		"/api/v1/group/:group_id/leave",
		r(crate::group::leave_group)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.delete(
		"/api/v1/group/:group_id/kick/:user_id",
		r(crate::group::kick_user_from_group)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/group/:group_id/invite/:invited_user",
		r(crate::group::invite_request)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/group/:group_id/invite_auto/:invited_user",
		r(crate::group::invite_auto)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/group/:group_id/invite/session/:key_session_id",
		r(crate::group::insert_user_keys_via_session_invite)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/group/:group_id/change_rank",
		r(crate::group::change_rank)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.patch(
		"/api/v1/group/:group_id/change_invite",
		r(crate::group::stop_invite)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/group/:group_id/join_req/:last_fetched_time/:last_user_id",
		r(crate::group::get_join_req)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/group/:group_id/join_req/:join_user",
		r(crate::group::accept_join_req)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.delete(
		"/api/v1/group/:group_id/join_req/:join_user",
		r(crate::group::reject_join_req)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/group/:group_id/join_req/session/:key_session_id",
		r(crate::group::insert_user_keys_via_session_join_req)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.post(
		"/api/v1/group/:group_id/key_rotation",
		r(crate::group::start_key_rotation)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.get(
		"/api/v1/group/:group_id/key_rotation",
		r(crate::group::get_keys_for_update)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
	router.put(
		"/api/v1/group/:group_id/key_rotation/:key_id",
		r(crate::group::done_key_rotation_for_user)
			.add(group::group_transform)
			.add(jwt::jwt_transform)
			.add(app_token::app_token_transform),
	);
}

/**
# Generated route files by rustgram route builder.

Please do not modify this file. Any changes will be overridden by the next route build.
Use the returned router instead
 */
use rustgram::{r, Router};

use crate::middleware::*;

pub(crate) fn routes() -> Router
{
	let mut router = Router::new(crate::not_found_handler);
	router.post("/api/v1/customer/register", r(crate::customer::register));
	router.get(
		"/api/v1/customer/register/:email_key",
		r(crate::customer::done_register),
	);
	router.post("/api/v1/customer/app", r(crate::customer_app::create_app));
	router.put("/api/v1/customer/app/:app_id", r(crate::customer_app::update));
	router.delete("/api/v1/customer/app/:app_id", r(crate::customer_app::delete));
	router.patch(
		"/api/v1/customer/app/:app_id/token_renew",
		r(crate::customer_app::renew_tokens),
	);
	router.patch(
		"/api/v1/customer/app/:app_id/new_jwt_keys",
		r(crate::customer_app::add_jwt_keys),
	);
	router.get(
		"/api/v1/customer/app/:app_id/jwt",
		r(crate::customer_app::get_jwt_details),
	);
	router.delete(
		"/api/v1/customer/app/:app_id/jwt/:jwt_id",
		r(crate::customer_app::delete_jwt_keys),
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
	router.get("/api/v1/user", r(crate::user::get).add(jwt::jwt_transform));
	router.put("/api/v1/user", r(crate::user::update).add(jwt::jwt_transform));
	router.put(
		"/api/v1/user/update_pw",
		r(crate::user::change_password).add(jwt::jwt_transform),
	);
	router.put(
		"/api/v1/user/reset_pw",
		r(crate::user::reset_password).add(jwt::jwt_transform),
	);
	router.delete(
		"/api/v1/user",
		r(crate::user::delete)
			.add(app_token::app_token_transform)
			.add(jwt::jwt_transform),
	);
	router.post(
		"/api/v1/group",
		r(crate::group::create)
			.add(app_token::app_token_transform)
			.add(jwt::jwt_transform),
	);
	router.delete(
		"/api/v1/group/:group_id",
		r(crate::group::delete)
			.add(app_token::app_token_transform)
			.add(jwt::jwt_transform),
	);
	router.get(
		"/api/v1/group/:group_id",
		r(crate::group::get).add(jwt::jwt_transform),
	);

	router
}

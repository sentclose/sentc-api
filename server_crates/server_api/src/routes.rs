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
	router.post("/api/v1/exists", r(crate::user::exists));
	router.post("/api/v1/register", r(crate::user::register));
	router.get("/api/v1/user", r(crate::user::get).add(jwt::jwt_transform));
	router.get("/api/v1/group/:id", r(crate::group::get).add(jwt::jwt_transform));

	router
}

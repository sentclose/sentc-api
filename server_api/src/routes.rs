/**
# Generated route files by rustgram route builder.

Please do not modify this file. Any changes will be overridden by the next route build.
Use the returned router instead
 */
use rustgram::{r, Router};

use crate::middleware::*;
use crate::user::*;

pub(crate) fn routes() -> Router
{
	let mut router = Router::new(crate::not_found_handler);
	router.get("/api/v1/user", r(crate::user::user_controller::get));
	router.post("/api/v1/user/exists", r(crate::user::user_controller::exists));

	router
}

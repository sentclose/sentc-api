use rustgram::{r, Request, Router};

use crate::routes::routes;

mod customer;
mod customer_app;
mod group;
mod key_management;
mod middleware;
mod routes;
mod user;
pub mod util;

pub use customer_app::app_entities::*;
#[cfg(feature = "embedded")]
pub use customer_app::app_service as sentc_customer_app_service;
#[cfg(feature = "embedded")]
pub use group::group_user_service as sentc_group_user_service;
#[cfg(feature = "embedded")]
pub use user::user_service as sentc_user_service;

async fn not_found_handler(_req: Request) -> util::api_res::JRes<String>
{
	Err(util::api_res::HttpErr::new(
		404,
		util::api_res::ApiErrorCodes::PageNotFound,
		"Not found".into(),
		None,
	))
}

async fn index_handler(_req: Request) -> &'static str
{
	"Hello there"
}

pub async fn start()
{
	//load the env
	dotenv::dotenv().ok();

	server_core::db::init_db().await;
	server_core::cache::init_cache().await;

	server_core::email::init_email_checker().await;
	#[cfg(feature = "send_mail")]
	server_core::email::send_mail::init_email_register().await;
}

/**
Entrypoint for every app

Use everytime, in standalone and other modes to get the router.

The other crates can use this router
 */
pub fn rest_routes() -> Router
{
	let mut router = routes();

	router.get("/", r(index_handler));

	router
}

use hyper::Body;
use rustgram::{r, Request, Response, Router};

use crate::routes::routes;

mod customer;
mod customer_app;
mod file;
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
pub use file::file_controller as sentc_file_controller;
#[cfg(feature = "embedded")]
pub use file::file_service as sentc_file_service;
#[cfg(feature = "embedded")]
pub use group::group_user_service as sentc_group_user_service;
#[cfg(feature = "embedded")]
pub use middleware::app_token::app_token_transform as sentc_app_mw;
#[cfg(feature = "embedded")]
pub use middleware::group::group_transform as sentc_group_mw;
#[cfg(feature = "embedded")]
pub use middleware::jwt::jwt_optional_transform as sentc_jwt_optional_mw;
#[cfg(feature = "embedded")]
pub use middleware::jwt::jwt_transform as sentc_jwt_mw;
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

async fn cors_handler(_req: Request) -> Response
{
	hyper::Response::builder()
		.header("Content-Length", "0")
		.header(
			"Access-Control-Allow-Methods",
			"GET, POST, PUT, DELETE, OPTIONS, PATCH",
		)
		.header("Access-Control-Max-Age", "86400")
		.header("Access-Control-Allow-Origin", "*")
		.header("Access-Control-Allow-Credentials", "true")
		.header(
			"Access-Control-Allow-Headers",
			"x-sentc-app-token, Content-Type, Accept, Origin, Authorization",
		)
		.body(Body::from(""))
		.unwrap()
}

pub async fn start()
{
	//load the env
	dotenv::dotenv().ok();

	server_core::db::init_db().await;
	server_core::cache::init_cache().await;
	server_core::file::init_storage().await;

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
	let mut router = Router::new(not_found_handler);

	routes(&mut router);

	router.get("/", r(index_handler));

	//cors route
	router.options("/*all", r(cors_handler));

	router
}

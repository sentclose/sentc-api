#![allow(clippy::too_many_arguments, clippy::manual_map, clippy::tabs_in_doc_comments, clippy::from_over_into)]

use hyper::Body;
use rustgram::{r, Request, Response, Router};

use crate::routes::routes;

mod content_management;
mod content_searchable;
mod customer;
mod customer_app;
mod file;
mod group;
mod key_management;
mod middleware;
mod routes;
mod user;
pub mod util;

pub use content_management::{
	content_controller as sentc_content_controller,
	content_entity as sentc_content_entities,
	content_service as sentc_content_service,
};
pub use content_searchable::{
	searchable_controller as sentc_searchable_controller,
	searchable_entities as sentc_searchable_entities,
	searchable_service as sentc_searchable_service,
};
pub use customer::{customer_controller as sentc_customer_controller, customer_entities as sentc_customer_entities};
pub use customer_app::{
	app_controller as sentc_app_controller,
	app_entities as sentc_app_entities,
	app_service as sentc_customer_app_service,
	app_util as sentc_app_utils,
};
pub use file::{file_controller as sentc_file_controller, file_service as sentc_file_service, file_worker as sentc_file_worker};
pub use group::{
	get_group_user_data_from_req,
	group_controller as sentc_group_controller,
	group_entities as sentc_group_entities,
	group_key_rotation_controller as sentc_group_key_rotation_controller,
	group_service as sentc_group_service,
	group_user_controller as sentc_group_user_controller,
	group_user_service as sentc_group_user_service,
	GROUP_TYPE_NORMAL,
	GROUP_TYPE_USER,
};
pub use key_management::{key_controller as sentc_key_controller, key_entity as sentc_key_entities};
pub use middleware::app_token::app_token_transform as sentc_app_mw;
pub use middleware::group::group_transform as sentc_group_mw;
pub use middleware::jwt::{jwt_optional_transform as sentc_jwt_optional_mw, jwt_transform as sentc_jwt_mw};
pub use user::{
	jwt as sentc_user_jwt_service,
	user_controller as sentc_user_controller,
	user_entities as sentc_user_entities,
	user_service as sentc_user_service,
};

pub async fn not_found_handler(_req: Request) -> util::api_res::JRes<String>
{
	Err(util::api_res::HttpErr::new(
		404,
		util::api_res::ApiErrorCodes::PageNotFound,
		"Not found".into(),
		None,
	))
}

pub async fn index_handler(_req: Request) -> &'static str
{
	"Hello there"
}

pub async fn cors_handler(_req: Request) -> Response
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
			"x-sentc-app-token, x-sentc-group-access-id, Content-Type, Accept, Origin, Authorization",
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

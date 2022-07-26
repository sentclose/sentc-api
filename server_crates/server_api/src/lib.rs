use rustgram::{r, Request, Router};

use crate::routes::routes;

pub mod core;
mod customer;
mod customer_app;
mod group;
mod middleware;
mod routes;
mod user;

async fn not_found_handler(_req: Request) -> core::api_res::JRes<String>
{
	Err(core::api_res::HttpErr::new(
		404,
		core::api_res::ApiErrorCodes::PageNotFound,
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

	core::db::init_db().await;
	core::cache::init_cache().await;
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

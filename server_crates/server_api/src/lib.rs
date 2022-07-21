use hyper::StatusCode;
use rustgram::{r, Request, Response, Router};

use crate::routes::routes;

pub mod core;
mod middleware;
mod routes;
mod user;

async fn not_found_handler(_req: Request) -> Response
{
	return hyper::Response::builder()
		.status(StatusCode::NOT_FOUND)
		.body("Not found".into())
		.unwrap();
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

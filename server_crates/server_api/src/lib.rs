use std::env;

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

/**
Entrypoint for every app

Use everytime, in standalone and other modes to get the router.

The other crates can use this router
*/
pub async fn start() -> Router
{
	//load the env
	dotenv::dotenv().ok();

	core::db::init_db().await;

	let mut router = routes();

	router.get("/", r(index_handler));

	router
}

/**
To start the Rest Api when running in standalone mode
*/
pub async fn start_app()
{
	let router = start().await;

	let addr = format!("{}:{}", env::var("SERVER_HOST").unwrap(), env::var("SERVER_PORT").unwrap())
		.parse()
		.unwrap();

	rustgram::start(router, addr).await;
}

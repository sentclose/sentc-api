use std::env;

use rustgram::{r, Request, Response, Router};
use rustgram_server_util::file::LocalStorage;
use rustgram_server_util::simple_static_server::{read_file_from_route_path, LOCAL_FILE_HANDLER};

pub async fn start()
{
	let path = env::var("LOCAL_FRONTED_DIR").unwrap_or_else(|_| "dist".to_string());

	LOCAL_FILE_HANDLER
		.get_or_init(move || async { LocalStorage::new(path) })
		.await;
}

/**
A Route to load the static files
 */
pub fn routes(router: &mut Router)
{
	router.get("/dashboard", r(read_file));
	router.get("/dashboard/", r(read_file));
	router.get("/dashboard/*path", r(read_file));
}

/**
Load the static file from the static dir.
 */
async fn read_file(req: Request) -> Response
{
	read_file_from_route_path(req, "path", Some("31536000")).await
}

use std::env;

use rustgram::{r, Router};
use server_api_common::{cors_handler, index_handler, not_found_handler, start};
use server_api_file::file_routes;

#[tokio::main]
pub async fn main()
{
	start().await;

	let mut router = Router::new(not_found_handler);

	file_routes(&mut router);

	router.get("/", r(index_handler));

	//cors route
	router.options("/*all", r(cors_handler));

	let addr = format!(
		"{}:{}",
		env::var("SERVER_HOST").unwrap(),
		env::var("SERVER_PORT").unwrap()
	)
	.parse()
	.unwrap();

	rustgram::start(router, addr).await;
}

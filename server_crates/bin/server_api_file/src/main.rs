use std::env;

use rustgram::{r, Router};
use server_api::{cors_handler, index_handler, not_found_handler, start};

#[tokio::main]
pub async fn main()
{
	start().await;

	let mut router = Router::new(not_found_handler);

	server_api_file::routes(&mut router);

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

use std::env;

use server_api::{rest_routes, start};
use server_bin::file::file_routes;
use server_bin::server_dashboard_backend;

/**
merge every server into one
 */
#[tokio::main]
pub async fn main()
{
	start().await;
	server_dashboard_backend::start().await;

	//routes from the rest api
	let mut router = rest_routes();

	//the static files from the dash board
	server_dashboard_backend::routes(&mut router);

	file_routes(&mut router);

	let addr = format!(
		"{}:{}",
		env::var("SERVER_HOST").unwrap(),
		env::var("SERVER_PORT").unwrap()
	)
	.parse()
	.unwrap();

	rustgram::start(router, addr).await;
}

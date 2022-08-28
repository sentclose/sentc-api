use std::env;

use server_api::{rest_routes, start};

/**
merge every server into one
*/
#[tokio::main]
pub async fn main()
{
	start().await;
	//routes from the rest api
	let mut router = rest_routes();

	//the static files from the dash board
	server_dashboard::routes(&mut router);

	server_api_file::routes(&mut router);

	let addr = format!(
		"{}:{}",
		env::var("SERVER_HOST").unwrap(),
		env::var("SERVER_PORT").unwrap()
	)
	.parse()
	.unwrap();

	rustgram::start(router, addr).await;
}

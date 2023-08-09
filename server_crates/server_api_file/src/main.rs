use std::env;

use server_api_file::file_routes;

#[tokio::main]
pub async fn main()
{
	server_api_common::start().await;

	let mut router = server_api_common::rest_routes();

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

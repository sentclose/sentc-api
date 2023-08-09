use std::env;

use server_api_customer::customer_routes;

#[tokio::main]
pub async fn main()
{
	server_api_customer::start().await;

	let mut router = server_api_common::rest_routes();

	customer_routes(&mut router);

	let addr = format!(
		"{}:{}",
		env::var("SERVER_HOST").unwrap(),
		env::var("SERVER_PORT").unwrap()
	)
	.parse()
	.unwrap();

	rustgram::start(router, addr).await;
}

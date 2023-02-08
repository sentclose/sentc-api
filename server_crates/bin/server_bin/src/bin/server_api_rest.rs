use std::env;

use server_api::{rest_routes, start};

#[tokio::main]
pub async fn main()
{
	start().await;
	let router = rest_routes();

	let addr = format!(
		"{}:{}",
		env::var("SERVER_HOST").unwrap(),
		env::var("SERVER_PORT").unwrap()
	)
	.parse()
	.unwrap();

	rustgram::start(router, addr).await;
}

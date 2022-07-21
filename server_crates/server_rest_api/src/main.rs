use std::env;

use server_api::start;

#[tokio::main]
pub async fn main()
{
	let router = start().await;

	let addr = format!("{}:{}", env::var("SERVER_HOST").unwrap(), env::var("SERVER_PORT").unwrap())
		.parse()
		.unwrap();

	rustgram::start(router, addr).await;
}

use std::env;

use rustgram::{Request, Router};
use server_api::{start, util};

async fn not_found_handler(_req: Request) -> util::api_res::JRes<String>
{
	Err(util::api_res::HttpErr::new(
		404,
		util::api_res::ApiErrorCodes::PageNotFound,
		"Not found".into(),
		None,
	))
}

#[tokio::main]
pub async fn main()
{
	start().await;

	let mut router = Router::new(not_found_handler);

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

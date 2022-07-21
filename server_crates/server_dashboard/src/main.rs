use std::env;

use rustgram::{Request, Router};
use server_dashboard::routes;

async fn not_found_handler(_req: Request) -> &'static str
{
	"Not found"
}

/**
The standalone version of the static file loader for the web dashboard

No db needed here.
*/
#[tokio::main]
pub async fn main()
{
	//load the env
	dotenv::dotenv().ok();

	let mut router = Router::new(not_found_handler);

	routes(&mut router);

	let addr = format!("{}:{}", env::var("SERVER_HOST").unwrap(), env::var("SERVER_PORT").unwrap())
		.parse()
		.unwrap();

	rustgram::start(router, addr).await;
}

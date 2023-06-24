use std::env;
use std::time::Duration;

use server_api::sentc_customer_app_service;

const INTERVAL_SEC: u64 = 60 * 30;

#[tokio::main]
async fn main()
{
	server_api::start().await;

	let demo_app_id = env::var("SENTC_APP_DEMO_ID").unwrap();

	let mut interval = tokio::time::interval(Duration::from_secs(INTERVAL_SEC));

	loop {
		interval.tick().await;

		println!("reset demo worker worker started");

		tokio::spawn(sentc_customer_app_service::reset(demo_app_id.clone()));
	}
}

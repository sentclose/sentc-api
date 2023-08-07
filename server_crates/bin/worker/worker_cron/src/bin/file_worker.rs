use std::time::Duration;

const INTERVAL_SEC: u64 = 60 * 60;

#[tokio::main]
async fn main()
{
	server_api_common::start().await;

	let mut interval = tokio::time::interval(Duration::from_secs(INTERVAL_SEC));

	loop {
		interval.tick().await;

		println!("file worker started");

		tokio::spawn(server_api_file::file_worker::start());
	}
}

use std::env;

use server_api::sentc_file_worker;

#[tokio::main]
async fn main()
{
	let args: Vec<String> = env::args().collect();

	server_api::start().await;

	match args[1].as_str() {
		"file" => sentc_file_worker::start().await.unwrap(),
		_ => panic!("Wrong args, please choose file"),
	}
}

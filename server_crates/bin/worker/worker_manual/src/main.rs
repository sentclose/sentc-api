use std::env;

#[tokio::main]
async fn main()
{
	let args: Vec<String> = env::args().collect();

	server_api_common::start().await;

	match args[1].as_str() {
		"file" => server_api_file::file_worker::start().await.unwrap(),
		_ => panic!("Wrong args, please choose file"),
	}
}

use std::env;

use rustgram::route_parser;

fn main()
{
	let args: Vec<String> = env::args().collect();

	match args[1].as_str() {
		"main" => main_routes(),
		"file" => file_routes(),
		"customer" => customer_routes(),
		_ => panic!("Wrong args, please choose main, customer or file"),
	}
}

fn main_routes()
{
	route_parser::start(
		"server_crates/server_api/routes.yml".to_string(),
		"server_crates/server_api/src/routes.rs".to_string(),
	);
}

fn file_routes()
{
	route_parser::start(
		"server_crates/server_api_file/routes.yml".to_string(),
		"server_crates/server_api_file/src/routes.rs".to_string(),
	);
}

fn customer_routes()
{
	route_parser::start(
		"server_crates/server_api_customer/routes.yml".to_string(),
		"server_crates/server_api_customer/src/routes.rs".to_string(),
	);
}

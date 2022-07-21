use rustgram::route_parser;

fn main()
{
	route_parser::start(
		"server_crates/server_api/routes.yml".to_string(),
		"server_crates/server_api/src/routes.rs".to_string(),
	);
}

use std::env;

use server_api_common::app::{AppFileOptionsInput, AppOptions, AppRegisterInput};

/**
Creates new tokens for the base sentc app, to manage the customer mod.

Only works with running api server.
*/

#[tokio::main]
async fn main()
{
	let args: Vec<String> = env::args().collect();

	server_api::start().await;

	match args[1].as_str() {
		"main_app" => main_app().await,
		"local_test" => test_app().await,
		_ => panic!("Wrong args, please choose app or local_test"),
	}
}

async fn main_app()
{
	server_api::sentc_customer_app_service::create_sentc_root_app()
		.await
		.unwrap();
}

async fn test_app()
{
	let input = AppRegisterInput {
		identifier: Some("test_app".to_string()),
		options: AppOptions::default_lax(),
		file_options: Default::default(),
		group_options: Default::default(),
	};

	let app_data = server_api::sentc_customer_app_service::create_app(input, "sentc_test".to_string(), None::<String>)
		.await
		.unwrap();

	println!("test app id: {}", app_data.app_id);
	println!("test secret_token: {}", app_data.secret_token);
	println!("test public_token: {}", app_data.public_token);
}

use std::env;

use server_api_common::app::{AppFileOptions, AppOptions, AppRegisterInput};

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
	//strict app options -> only to create the customer register app
	let input = AppRegisterInput {
		identifier: None,
		options: AppOptions {
			group_create: 0,
			group_get: 0,
			group_user_keys: 0,
			group_user_update_check: 0,
			group_invite: 0,
			group_auto_invite: 0,
			group_reject_invite: 0,
			group_accept_invite: 0,
			group_join_req: 0,
			group_accept_join_req: 0,
			group_reject_join_req: 0,
			group_key_rotation: 0,
			group_user_delete: 0,
			group_delete: 0,
			group_leave: 0,
			group_change_rank: 0,
			user_exists: 0,
			user_register: 0,
			user_delete: 0,
			user_update: 0,
			user_change_password: 0,
			user_reset_password: 0,
			user_prepare_login: 0,
			user_done_login: 0,
			user_public_data: 0,
			user_jwt_refresh: 0,
			key_register: 0,
			key_get: 0,
			group_list: 0,
			file_register: 0,
			file_part_upload: 0,
			file_get: 0,
			file_part_download: 0,
			user_device_register: 0,
			user_device_delete: 0,
		},
		file_options: AppFileOptions {
			file_storage: -1,
			storage_url: None,
		},
	};

	let app_data = server_api::sentc_customer_app_service::create_app(input, "sentc_init".to_string())
		.await
		.unwrap();

	println!("app id: {}", app_data.app_id);
	println!("secret_token: {}", app_data.secret_token);
	println!("public_token: {}", app_data.public_token);
}

async fn test_app()
{
	let input = AppRegisterInput {
		identifier: Some("test_app".to_string()),
		options: AppOptions::default_lax(),
		file_options: Default::default(),
	};

	let app_data = server_api::sentc_customer_app_service::create_app(input, "sentc_test".to_string())
		.await
		.unwrap();

	println!("test app id: {}", app_data.app_id);
	println!("test secret_token: {}", app_data.secret_token);
	println!("test public_token: {}", app_data.public_token);
}

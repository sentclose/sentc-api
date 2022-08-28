use server_api_common::app::{AppOptions, AppRegisterInput};

/**
Creates new tokens for the base sentc app, to manage the customer mod.

Only works with running api server.
*/

#[tokio::main]
async fn main()
{
	server_api::start().await;

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
		},
	};

	let app_data = server_api::sentc_customer_app_service::create_app(input, "sentc_init".to_string())
		.await
		.unwrap();

	println!("app id: {}", app_data.app_id);
	println!("secret_token: {}", app_data.secret_token);
	println!("public_token: {}", app_data.public_token);
}

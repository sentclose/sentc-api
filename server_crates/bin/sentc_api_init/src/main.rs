use server_api::SENTC_ROOT_APP;

#[tokio::main]
async fn main()
{
	server_api::start().await;

	//check the server root app
	let root_exists = server_api::sentc_customer_app_service::check_app_exists(SENTC_ROOT_APP, SENTC_ROOT_APP)
		.await
		.unwrap();

	if !root_exists {
		println!("Root app not exists, trying to create it, please wait.");
		server_api::sentc_customer_app_service::create_sentc_root_app()
			.await
			.unwrap();
		println!("Root app successfully created.");
	}
}

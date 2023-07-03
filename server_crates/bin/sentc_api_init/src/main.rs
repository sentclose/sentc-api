use std::env;
use std::fs::File;
use std::io::Read;

use rustgram_server_util::db::{exec_string_non_param, query_first_non_param, StringEntity};
use server_api::SENTC_ROOT_APP;

#[tokio::main]
async fn main()
{
	let args: Vec<String> = env::args().collect();

	server_api::start().await;

	if let Some(a) = args.get(1) {
		//if arg set then do the requested
		match a.as_str() {
			"db" => {
				check_db().await;
			},
			"root" => {
				create_root_app().await;
			},
			_ => panic!("Wrong args, please choose `db` or `root`"),
		}
	} else {
		//if no args set -> do everything
		check_db().await;

		create_root_app().await;
	}
}

async fn create_root_app()
{
	//check the server root app

	let root_exists = server_api::sentc_customer_app_service::check_app_exists(SENTC_ROOT_APP, SENTC_ROOT_APP)
		.await
		.unwrap();

	if !root_exists {
		println!("--------");
		println!("Root app not exists, trying to create it, please wait.");
		server_api::sentc_customer_app_service::create_sentc_root_app()
			.await
			.unwrap();
		println!("Root app successfully created.");
	}
}

#[cfg(feature = "mysql")]
async fn check_db()
{
	//language=SQL
	let sql = "SHOW TABLES LIKE 'sentc_app'";

	let res: Option<StringEntity> = query_first_non_param(sql).await.unwrap();

	if res.is_none() {
		println!("--------");
		println!("Database tables not found. Running migration, please wait.");
		migrate_db().await;
		println!("Database migration was successfully.");
	}
}

#[cfg(feature = "sqlite")]
async fn check_db()
{
	//language=SQLx
	let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'sentc_app'";

	let res: Option<StringEntity> = query_first_non_param(sql).await.unwrap();

	if res.is_none() {
		println!("--------");
		println!("Database tables not found. Running migration, please wait.");
		migrate_db().await;
		println!("Database migration was successfully.");
	}
}

#[cfg(feature = "mysql")]
async fn migrate_db()
{
	let mut file = File::open("db/migrations/mysql/complete/sentc_mysql.sql").unwrap();

	let mut sql = String::new();

	file.read_to_string(&mut sql).unwrap();

	exec_string_non_param(sql).await.unwrap();
}

#[cfg(feature = "sqlite")]
async fn migrate_db()
{
	let mut file = File::open("db/migrations/sqlite/complete/sentc_sqlite.sql").unwrap();

	let mut sql = String::new();

	file.read_to_string(&mut sql).unwrap();

	exec_string_non_param(sql).await.unwrap();
}

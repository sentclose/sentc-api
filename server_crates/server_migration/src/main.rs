use std::env;

#[tokio::main]
async fn main()
{
	//load the env
	dotenv::dotenv().ok();

	server_api::core::db::init_db().await;

	let args: Vec<String> = env::args().collect();

	match args[1].as_str() {
		"up" => up().await,
		"down" => down().await,
		_ => panic!("Wrong args for db migrate"),
	}
}

/**
TODO

- get the actual db version in the db table version
- check if there is a newer version in the migrate directory
	(by reading the dir, can be done with std, because we are not running a server)
- if not: do nothing
- if:
- execute every sql file until the last version
*/
async fn up()
{
	println!("calling from up");
}

/**
TODO

- get the actual db version in the db table version
- check if there is an older version in the migrate directory
if not: do nothing
-if: execute the last sql file in the down dir
*/
async fn down()
{
	println!("calling from down");
}

mod core;
mod user;

pub async fn start()
{
	//load the env
	dotenv::dotenv().ok();

	core::db::init_db().await;
}

use deadpool_sqlite::{Config, Pool, Runtime};

use crate::core::api_err::{ApiErrorCodes, HttpErr};
use crate::core::db::SQLITE_DB_CONN;

pub async fn create_db() -> Pool
{
	let cfg = Config::new("db/sqlite/db.sqlite3");
	let pool = cfg.create_pool(Runtime::Tokio1).unwrap();
	let conn = pool.get().await.unwrap();

	let result: i64 = conn
		.interact(|conn| {
			//test db connection
			let mut stmt = conn.prepare("SELECT 1")?;
			let mut rows = stmt.query([])?;
			let row = rows.next()?.unwrap();
			row.get(0)
		})
		.await
		.unwrap()
		.unwrap();

	assert_eq!(result, 1);

	println!("init sqlite");

	pool
}

pub async fn get_conn() -> Result<deadpool_sqlite::Object, HttpErr>
{
	match SQLITE_DB_CONN.get().unwrap().get().await {
		Ok(c) => Ok(c),
		Err(e) => {
			Err(HttpErr::new(
				500,
				ApiErrorCodes::NoDbConnection,
				"No db connection",
				Some(format!("db connection error for sqlite: {:?}", e)),
			))
		},
	}
}

use deadpool_sqlite::{Config, Pool, Runtime};
use rusqlite::{params_from_iter, Connection, Row, ToSql};

use crate::core::api_err::{ApiErrorCodes, HttpErr};
use crate::core::db::{db_exec_err, SQLITE_DB_CONN};

pub trait FromSqliteRow
{
	fn from_row_opt(row: &Row) -> Result<Self, HttpErr>
	where
		Self: Sized;
}

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

fn exec_sync<T, P>(conn: &mut Connection, sql: &str, params: P) -> Result<Vec<T>, HttpErr>
where
	T: FromSqliteRow,
	P: IntoIterator,
	P::Item: ToSql,
{
	let mut stmt = conn.prepare(sql).map_err(|e| db_exec_err(&e))?;

	let mut rows = stmt
		.query(params_from_iter(params))
		.map_err(|e| db_exec_err(&e))?;

	let mut init: Vec<T> = Vec::new();

	while let Some(row) = rows.next().map_err(|e| db_exec_err(&e))? {
		init.push(FromSqliteRow::from_row_opt(row)?)
	}

	Ok(init)
}

/**
# Execute and fetch from db

````ignore
use rusqlite::Row;

pub struct Lol
{
	pub lol: String,
	pub lol_count: i32,
}

impl FromSqliteRow for Lol
{
	fn from_row_opt(row: &Row) -> Result<Self, HttpErr>
	where
		Self: Sized,
	{
		Ok(Lol {
			lol: row.get(0).map_err(|e| db_exec_err(&e))?,
			lol_count: row.get(1).map_err(|e| db_exec_err(&e))?,
		})
	}
}


async fn lol()
{
	//language=SQL
	let sql = "SELECT 1";
	let params = crate::set_params!("1".to_string(), 2_i32);

	let lol = exec::<Lol, _>(sql, params).await.unwrap();

	//or from a vec (every item must be the same type for vec

	let param_vec = vec!["123".to_string(), "1".to_string()];

	let lol = exec::<Lol, _>(sql, param_vec).await.unwrap();
}

````
*/
pub async fn exec<T, P>(sql: &'static str, params: P) -> Result<Vec<T>, HttpErr>
where
	T: FromSqliteRow + Send + 'static,
	P: IntoIterator + Send + 'static,
	P::Item: ToSql,
{
	let conn = get_conn().await?;

	let result = conn
		.interact(move |conn| exec_sync::<T, P>(conn, sql, params))
		.await
		.map_err(|e| db_exec_err(&e))??;

	Ok(result)
}

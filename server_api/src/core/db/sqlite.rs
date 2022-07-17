use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use deadpool_sqlite::{Config, Pool, Runtime};
use rusqlite::{params_from_iter, Connection, Row, ToSql};

use crate::core::api_err::{ApiErrorCodes, HttpErr};
use crate::core::db::{db_exec_err, db_query_err, SQLITE_DB_CONN};

#[derive(Debug)]
pub struct FormSqliteRowError
{
	pub msg: String,
}

impl Error for FormSqliteRowError {}

impl Display for FormSqliteRowError
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
	{
		write!(f, "Err in db fetch: {}", self.msg)
	}
}

pub trait FromSqliteRow
{
	fn from_row_opt(row: &Row) -> Result<Self, FormSqliteRowError>
	where
		Self: Sized;
}

pub async fn create_db() -> Pool
{
	let cfg = Config::new(std::env::var("DB_PATH").unwrap());
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

fn query_sync<T, P>(conn: &mut Connection, sql: &str, params: P) -> Result<Vec<T>, HttpErr>
where
	T: FromSqliteRow,
	P: IntoIterator,
	P::Item: ToSql,
{
	let mut stmt = conn.prepare(sql).map_err(|e| db_query_err(&e))?;

	let mut rows = stmt
		.query(params_from_iter(params))
		.map_err(|e| db_query_err(&e))?;

	let mut init: Vec<T> = Vec::new();

	while let Some(row) = rows.next().map_err(|e| db_query_err(&e))? {
		init.push(FromSqliteRow::from_row_opt(row).map_err(|e| db_query_err(&e))?)
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
	fn from_row_opt(row: &Row) -> Result<Self, FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Lol {
			lol: take_or_err(row, 0),
			lol_count: take_or_err(row, 1,
		})
	}
}


async fn lol()
{
	//language=SQL
	let sql = "SELECT 1";
	let params = crate::set_params!("1".to_string(), 2_i32);

	let lol = query::<Lol, _>(sql, params).await.unwrap();

	//or from a vec (every item must be the same type for vec

	let param_vec = vec!["123".to_string(), "1".to_string()];

	let lol = query::<Lol, _>(sql, param_vec).await.unwrap();
}

````
*/
pub async fn query<T, P>(sql: String, params: P) -> Result<Vec<T>, HttpErr>
where
	T: FromSqliteRow + Send + 'static,
	P: IntoIterator + Send + 'static,
	P::Item: ToSql,
{
	let conn = get_conn().await?;

	let result = conn
		.interact(move |conn| query_sync::<T, P>(conn, sql.as_str(), params))
		.await
		.map_err(|e| db_query_err(&e))??;

	Ok(result)
}

fn exec_sync<P>(conn: &mut Connection, sql: &str, params: P) -> Result<usize, HttpErr>
where
	P: IntoIterator,
	P::Item: ToSql,
{
	conn.execute(sql, params_from_iter(params))
		.map_err(|e| db_exec_err(&e))
}

/**
# Executes an sql stmt

````ignore
async fn lol()
{
	//language=SQL
	let sql = "INSERT INTO table (col1, col2) VALUES (?,?)";
	let params = crate::set_params!("1".to_string(), 2_i32);

	let lol = exec(sql, params).await.unwrap();

	//or from a vec (every item must be the same type for vec

	let param_vec = vec!["123".to_string(), "1".to_string()];

	let lol = exec(sql, param_vec).await.unwrap();
}

````
*/
pub async fn exec<P>(sql: &'static str, params: P) -> Result<usize, HttpErr>
where
	P: IntoIterator + Send + 'static,
	P::Item: ToSql,
{
	let conn = get_conn().await?;

	let result = conn
		.interact(move |conn| exec_sync(conn, sql, params))
		.await
		.map_err(|e| db_exec_err(&e))??;

	Ok(result)
}

#[macro_export]
macro_rules! take_or_err {
	($row:expr, $index:expr) => {
		match $row.get($index) {
			Ok(v) => v,
			Err(e) => {
				return Err(crate::core::db::FormSqliteRowError {
					msg: format!("{:?}", e),
				})
			},
		}
	};
}

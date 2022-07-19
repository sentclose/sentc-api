use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use deadpool_sqlite::{Config, Pool, Runtime};
use rusqlite::{params_from_iter, Connection, Row, ToSql};

use crate::core::api_err::{ApiErrorCodes, HttpErr};
use crate::core::db::{db_bulk_insert_err, db_exec_err, db_query_err, SQLITE_DB_CONN};

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

fn query_first_sync<T, P>(conn: &mut Connection, sql: &str, params: P) -> Result<Option<T>, HttpErr>
where
	T: FromSqliteRow,
	P: IntoIterator,
	P::Item: ToSql,
{
	let mut stmt = conn.prepare(sql).map_err(|e| db_query_err(&e))?;

	let mut rows = stmt
		.query(params_from_iter(params))
		.map_err(|e| db_query_err(&e))?;

	match rows.next().map_err(|e| db_query_err(&e))? {
		Some(row) => Ok(Some(FromSqliteRow::from_row_opt(row).map_err(|e| db_query_err(&e))?)),
		None => Ok(None),
	}
}

pub async fn query_first<T, P>(sql: String, params: P) -> Result<Option<T>, HttpErr>
where
	T: FromSqliteRow + Send + 'static,
	P: IntoIterator + Send + 'static,
	P::Item: ToSql,
{
	let conn = get_conn().await?;

	let result = conn
		.interact(move |conn| query_first_sync::<T, P>(conn, sql.as_str(), params))
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

fn bulk_insert_sync<F, T>(conn: &mut Connection, ignore: bool, table: String, cols: Vec<String>, objects: &Vec<T>, fun: F) -> Result<usize, HttpErr>
where
	F: Fn(&T) -> Vec<rusqlite::types::Value>,
{
	//prepare the sql
	let ignore_string = if ignore { " OR IGNORE" } else { "" };

	let mut stmt = format!("INSERT {} INTO {} ({}) VALUES ", ignore_string, table, cols.join(","));
	// each (?,..,?) tuple for values
	let row = format!(
		"({}),",
		cols.iter()
			.map(|_| "?".to_string())
			.collect::<Vec<_>>()
			.join(",")
	);

	stmt.reserve(objects.len() * (cols.len() * 2 + 2));

	// add the row tuples in the query
	for _ in 0..objects.len() {
		stmt.push_str(&row);
	}

	// remove the trailing comma
	stmt.pop();

	let mut params = Vec::new();

	//using rustsqlite value https://stackoverflow.com/questions/69230495/how-to-pass-vecvalue-in-rusqlite-as-query-param
	for o in objects.iter() {
		for val in fun(o) {
			params.push(val);
		}
	}

	//transaction from here: https://github.com/avinassh/fast-sqlite3-inserts/blob/master/src/bin/basic.rs
	//but not necessary for inserting in one table
	let tx = conn.transaction().map_err(|e| db_bulk_insert_err(&e))?;

	let result = tx
		.execute(stmt.as_str(), params_from_iter(params))
		.map_err(|e| db_bulk_insert_err(&e))?;

	tx.commit().map_err(|e| db_bulk_insert_err(&e))?;

	Ok(result)
}

/**
# let insert multiple objets into the db

got it form here: https://github.com/blackbeam/rust-mysql-simple/issues/59#issuecomment-245918807

`T` is the object type

`fn` transformed the obj values to params

`ignore` do an insert ignore

creates a query like this:
```SQL
INSERT INTO table (fields...) VALUES (?, ?, ?), (?, ?, ?), (?, ?, ?), ...
```
 */
pub async fn bulk_insert<F: 'static + Send + Sync, T: 'static + Send + Sync>(
	ignore: bool,
	table: String,
	cols: Vec<String>,
	objects: Vec<T>, //must be pass by value because we need static lifetime here for the deadpool interact
	fun: F,
) -> Result<usize, HttpErr>
where
	F: Fn(&T) -> Vec<rusqlite::types::Value>,
{
	let conn = get_conn().await?;

	let res = conn
		.interact(move |conn| bulk_insert_sync(conn, ignore, table, cols, &objects, fun))
		.await
		.map_err(|e| db_bulk_insert_err(&e))??;

	Ok(res)
}

use std::env;

use mysql_async::prelude::{FromRow, Queryable};
use mysql_async::{Params, Pool, TxOpts};

use crate::db::{db_bulk_insert_err, db_exec_err, db_query_err, db_tx_err, MARIA_DB_COMM};
use crate::error::{CoreErrorCodes, SentcCoreError, SentcErrorConstructor};

#[macro_export]
macro_rules! take_or_err {
	($row:expr, $index:expr, $t:ident) => {
		match $row.take_opt::<$t, _>($index) {
			Some(value) => {
				match value {
					Ok(ir) => ir,
					Err(server_core::db::mysql_async_export::FromValueError(_value)) => {
						return Err(server_core::db::mysql_async_export::FromRowError($row));
					},
				}
			},
			None => return Err(server_core::db::mysql_async_export::FromRowError($row)),
		}
	};
	($row:expr, $index:expr, Option<$t:ident>) => {
		match $row.take_opt::<Option<$t>, _>($index) {
			Some(value) => {
				match value {
					Ok(ir) => ir,
					Err(server_core::db::mysql_async_export::FromValueError(_value)) => {
						return Err(server_core::db::mysql_async_export::FromRowError($row));
					},
				}
			},
			None => return Err(server_core::db::mysql_async_export::FromRowError($row)),
		}
	};
}

#[macro_export]
macro_rules! take_or_err_opt {
	($row:expr, $index:expr, $t:ident) => {
		match $row.take_opt::<Option<$t>, _>($index) {
			Some(value) => {
				match value {
					Ok(ir) => ir,
					Err(server_core::db::mysql_async_export::FromValueError(_value)) => {
						return Err(server_core::db::mysql_async_export::FromRowError($row));
					},
				}
			},
			None => return Err(server_core::db::mysql_async_export::FromRowError($row)),
		}
	};
}

pub struct TransactionData<'a, P>
where
	P: Into<Params> + Send,
{
	pub sql: &'a str,
	pub params: P,
}

pub async fn create_db() -> Pool
{
	let user = env::var("DB_USER").unwrap();
	let pw = env::var("DB_PASS").unwrap();
	let mysql_host = env::var("DB_HOST").unwrap();
	let db = env::var("DB_NAME").unwrap();

	let pool = Pool::new(format!("mysql://{}:{}@{}/{}", user, pw, mysql_host, db).as_str());

	println!("init mariadb");

	pool
}

pub async fn get_conn() -> Result<mysql_async::Conn, SentcCoreError>
{
	//get conn with a loop because for very much workload we getting an err -> try again
	let maria_db = MARIA_DB_COMM.get().unwrap();

	let mut i = 0; //say how much iteration should be done before giving up

	loop {
		if i > 10 {
			return Err(SentcCoreError::new_msg_and_debug(
				500,
				CoreErrorCodes::NoDbConnection,
				"No db connection",
				Some("No connection after 10 tries".to_owned()),
			));
		}

		match maria_db.get_conn().await {
			Ok(conn_ty) => {
				return Ok(conn_ty);
			},
			Err(_e) => {
				//println!("{:?}", e);
			},
		}

		i += 1;
	}
}

/**
# call mysql-async exec function

handles the err and return a `HttpErr` instead of the db err

so we can just use it like:
```ignore
//language=SQL
let sql = "SELECT tag_id, belongs_to, type FROM tags_belongs_to WHERE tag_id = ?";

// the , in ("lol",) is important!
//exec is from mysql_async
let result = exec::<TagsBelongsTo, _>(sql, ("lol",)).await?;

match to_string(&result) {
	Ok(o) => Ok(o),
	Err(e) => Err(HttpErr::new(422, 10, format!("db error"), Some(format!("db fetch err, {:?}", e)))),
}
```

instead of this (don't do this, no err handling here):
```ignore
//language=SQL
let sql = "SELECT tag_id, belongs_to, type FROM tags_belongs_to WHERE tag_id = ?";

let mut conn = get_conn().await?;

// the , in ("lol",) is important!
let result = conn
	.query::<TagsBelongsTo, _, _>(sql, ("lol",))
	.await
	.unwrap();

Ok(to_string(&result).unwrap())
```
 */
pub async fn query<T, P>(sql: &'static str, params: P) -> Result<Vec<T>, SentcCoreError>
where
	T: FromRow + Send + 'static,
	P: Into<Params> + Send,
{
	let mut conn = get_conn().await?;

	conn.exec::<T, _, P>(sql, params)
		.await
		.map_err(|e| db_query_err(&e))
}

/**
The same as query but sql with a string.

This is used to get the sql string from the get in fn
*/
pub async fn query_string<T, P>(sql: String, params: P) -> Result<Vec<T>, SentcCoreError>
where
	T: FromRow + Send + 'static,
	P: Into<Params> + Send,
{
	let mut conn = get_conn().await?;

	conn.exec::<T, _, P>(sql, params)
		.await
		.map_err(|e| db_query_err(&e))
}

/**
# Query and get the first result

No vec gets returned, but an options enum
*/
pub async fn query_first<T, P>(sql: &'static str, params: P) -> Result<Option<T>, SentcCoreError>
where
	T: FromRow + Send + 'static,
	P: Into<Params> + Send,
{
	let mut conn = get_conn().await?;

	conn.exec_first::<T, _, P>(sql, params)
		.await
		.map_err(|e| db_query_err(&e))
}

/**
The same as query but sql with a string.

This is used to get the sql string from the get in fn
 */
pub async fn query_first_string<T, P>(sql: String, params: P) -> Result<Option<T>, SentcCoreError>
where
	T: FromRow + Send + 'static,
	P: Into<Params> + Send,
{
	let mut conn = get_conn().await?;

	conn.exec_first::<T, _, P>(sql, params)
		.await
		.map_err(|e| db_query_err(&e))
}

/**
# Execute a sql stmt

drop the result just execute
*/
pub async fn exec<P>(sql: &str, params: P) -> Result<(), SentcCoreError>
where
	P: Into<Params> + Send,
{
	let mut conn = get_conn().await?;

	conn.exec_drop(sql, params)
		.await
		.map_err(|e| db_exec_err(&e))
}

pub async fn exec_string<P>(sql: String, params: P) -> Result<(), SentcCoreError>
where
	P: Into<Params> + Send,
{
	let mut conn = get_conn().await?;

	conn.exec_drop(sql, params)
		.await
		.map_err(|e| db_exec_err(&e))
}

/**
# Execute in transaction

can be multiple stmt with params in one transition
*/
pub async fn exec_transaction<P>(data: Vec<TransactionData<'_, P>>) -> Result<(), SentcCoreError>
where
	P: Into<Params> + Send,
{
	let mut conn = get_conn().await?;
	let mut tx = conn
		.start_transaction(TxOpts::default())
		.await
		.map_err(|e| db_tx_err(&e))?;

	for datum in data {
		tx.exec_drop(datum.sql, datum.params)
			.await
			.map_err(|e| db_tx_err(&e))?;
	}

	tx.commit().await.map_err(|e| db_tx_err(&e))
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
pub async fn bulk_insert<F, P, T>(ignore: bool, table: String, cols: Vec<String>, objects: Vec<T>, fun: F) -> Result<(), SentcCoreError>
where
	F: Fn(&T) -> P,
	P: Into<Params>,
{
	let ignore_string = if ignore { "IGNORE" } else { "" };

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

	for o in objects.iter() {
		let new_params: Params = fun(o).into();

		if let Params::Positional(new_params) = new_params {
			for param in new_params {
				params.push(param);
			}
		}
	}

	let mut conn = get_conn().await?;

	conn.exec_drop(stmt, params)
		.await
		.map_err(|e| db_bulk_insert_err(&e))
}

use std::error::Error;

use tokio::sync::OnceCell;

use crate::core::api_err::{ApiErrorCodes, HttpErr};

#[cfg(feature = "mysql")]
mod mariadb;
#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "mysql")]
pub use self::mariadb::{exec, query};
#[cfg(feature = "sqlite")]
pub use self::sqlite::{exec, query, FromSqliteRow};

#[cfg(feature = "sqlite")]
static SQLITE_DB_CONN: OnceCell<deadpool_sqlite::Pool> = OnceCell::const_new();

#[cfg(feature = "mysql")]
static MARIA_DB_COMM: OnceCell<mysql_async::Pool> = OnceCell::const_new();

pub async fn init_db()
{
	#[cfg(feature = "sqlite")]
	SQLITE_DB_CONN.get_or_init(sqlite::create_db).await;

	#[cfg(feature = "mysql")]
	MARIA_DB_COMM.get_or_init(mariadb::create_db).await;
}

/**
# Returns a ? string for multiple parameter

````rust_sample
	let ids = vec!["lol", "abc", "123"];

	let ins = get_in(&ids);

	println!("{:?}", ins);

	//prints "?,?,?"
````
 */
pub fn get_in<T>(objects: &Vec<T>) -> String
{
	format!(
		"{}",
		objects
			.iter()
			.map(|_| "?".to_string())
			.collect::<Vec<_>>()
			.join(",")
	)
}

fn db_query_err<E: Error>(e: &E) -> HttpErr
{
	HttpErr::new(422, ApiErrorCodes::DbQuery, "db error", Some(format!("db fetch err, {:?}", e)))
}

fn db_exec_err<E: Error>(e: &E) -> HttpErr
{
	HttpErr::new(422, ApiErrorCodes::DbExecute, "db error", Some(format!("db execute err, {:?}", e)))
}

/**
# Tuple for async-mysql params

returns a tuple of the input values.

 */
#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! set_params {
	($( $param:expr ),+ $(,)?) => {{
		($($param),+ ,)
	}};
}

/**
# The sql params for sqlite

 */
#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! set_params {
	($( $param:expr ),+ $(,)?) => {{
		let mut tmp = Vec::new();

		$(
			tmp.push(rusqlite::types::Value::from($param));
		)*

		tmp
	}};
}

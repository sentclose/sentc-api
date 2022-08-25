use std::error::Error;

use tokio::sync::OnceCell;

use crate::error::{CoreError, CoreErrorCodes};

#[cfg(feature = "mysql")]
mod mariadb;
#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "mysql")]
pub use self::mariadb::{bulk_insert, exec, exec_string, exec_transaction, query, query_first, query_first_string, query_string, TransactionData};
#[cfg(feature = "sqlite")]
pub use self::sqlite::{
	bulk_insert,
	exec,
	exec_string,
	exec_transaction,
	query,
	query_first,
	query_first_string,
	query_string,
	FormSqliteRowError,
	FromSqliteRow,
	TransactionData,
};

#[cfg(feature = "sqlite")]
static SQLITE_DB_CONN: OnceCell<deadpool_sqlite::Pool> = OnceCell::const_new();

#[cfg(feature = "mysql")]
static MARIA_DB_COMM: OnceCell<mysql_async::Pool> = OnceCell::const_new();

#[cfg(feature = "mysql")]
pub type Params = mysql_common::params::Params;

#[cfg(feature = "sqlite")]
pub type Params = Vec<rusqlite::types::Value>;

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

fn db_query_err<E: Error>(e: &E) -> CoreError
{
	CoreError::new(
		422,
		CoreErrorCodes::DbQuery,
		"db error".to_owned(),
		Some(format!("db fetch err, {:?}", e)),
	)
}

fn db_exec_err<E: Error>(e: &E) -> CoreError
{
	CoreError::new(
		422,
		CoreErrorCodes::DbExecute,
		"db error".to_owned(),
		Some(format!("db execute err, {:?}", e)),
	)
}

fn db_bulk_insert_err<E: Error>(e: &E) -> CoreError
{
	CoreError::new(
		422,
		CoreErrorCodes::DbBulkInsert,
		"db error".to_owned(),
		Some(format!("db bulk insert err, {:?}", e)),
	)
}

fn db_tx_err<E: Error>(e: &E) -> CoreError
{
	CoreError::new(
		422,
		CoreErrorCodes::DbTx,
		"Db error".to_owned(),
		Some(format!("Db transaction error: {:?}", e)),
	)
}

/**
# Tuple for async-mysql params

transform the values like into_params_impl from mysql_common::params

 */
#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! set_params {
	($( $param:expr ),+ $(,)?) => {{
		 mysql_common::params::Params::Positional(vec![
			 $($param.into(),)*
         ])
	}};
}

#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! set_params_vec {
	($vec:expr) => {{
		let mut out: Vec<mysql_common::value::Value> = Vec::with_capacity($vec.len());

		for inp in $vec {
			out.push(inp.0.into());
		}

		mysql_common::params::Params::Positional(out)
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

#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! set_params_vec {
	($vec:expr) => {{
		let mut tmp = Vec::with_capacity($vec.len());

		for inp in $vec {
			tmp.push(rusqlite::types::Value::from(inp.0))
		}

		tmp
	}};
}
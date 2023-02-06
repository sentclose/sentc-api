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

#[allow(clippy::useless_format)]
/**
# Returns a ? string for multiple parameter

````rust_sample
	let ids = vec!["lol", "abc", "123"];

	let ins = get_in(&ids);

	println!("{:?}", ins);

	//prints "?,?,?"
````
 */
pub fn get_in<T>(objects: &[T]) -> String
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
		 server_core::db::mysql_common_export::params::Params::Positional(vec![
			 $($param.into(),)*
         ])
	}};
}

#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! set_params_vec {
	($vec:expr) => {{
		let mut out: Vec<server_core::db::mysql_common_export::value::Value> = Vec::with_capacity($vec.len());

		for inp in $vec {
			out.push(inp.0.into());
		}

		server_core::db::mysql_common_export::params::Params::Positional(out)
	}};
}

#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! set_params_vec_outer {
	($vec:expr) => {{
		let mut out: Vec<server_core::db::mysql_common_export::value::Value> = Vec::with_capacity($vec.len());

		for inp in $vec {
			out.push(inp.into());
		}

		server_core::db::mysql_common_export::params::Params::Positional(out)
	}};
}

/**
# The sql params for sqlite

 */
#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! set_params {
	($( $param:expr ),+ $(,)?) => {
		vec![
			$(server_core::db::rusqlite_export::types::Value::from($param),)*
		]
	};
}

#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! set_params_vec {
	($vec:expr) => {{
		let mut tmp = Vec::with_capacity($vec.len());

		for inp in $vec {
			tmp.push(server_core::db::rusqlite_export::types::Value::from(inp.0))
		}

		tmp
	}};
}

#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! set_params_vec_outer {
	($vec:expr) => {{
		let mut tmp = Vec::with_capacity($vec.len());

		for inp in $vec {
			tmp.push(server_core::db::rusqlite_export::types::Value::from(inp))
		}

		tmp
	}};
}

#[cfg(feature = "mysql")]
pub use mysql_async as mysql_async_export;
#[cfg(feature = "mysql")]
pub use mysql_common as mysql_common_export;
#[cfg(feature = "sqlite")]
pub use rusqlite as rusqlite_export;

//__________________________________________________________________________________________________

//impl for one tuple structs

pub struct TupleEntity<T>(pub T);

#[cfg(feature = "mysql")]
impl<T: mysql_async::prelude::FromValue> mysql_async::prelude::FromRow for TupleEntity<T>
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self(match row.take_opt(0) {
			Some(value) => {
				match value {
					Ok(ir) => ir,
					Err(mysql_async::FromValueError(_value)) => return Err(mysql_async::FromRowError(row)),
				}
			},
			None => return Err(mysql_async::FromRowError(row)),
		}))
	}
}

#[cfg(feature = "sqlite")]
impl<T: rusqlite::types::FromSql> crate::db::FromSqliteRow for TupleEntity<T>
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(match row.get(0) {
			Ok(v) => v,
			Err(e) => {
				return Err(crate::db::FormSqliteRowError {
					msg: format!("{:?}", e),
				})
			},
		}))
	}
}

pub type StringEntity = TupleEntity<String>;

pub type I32Entity = TupleEntity<i32>;

pub type I64Entity = TupleEntity<i64>;

//__________________________________________________________________________________________________
//str handling. sqlite needs static ref or owned values because of the tokio spawn block

#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! str_t {
	() => {
		&str
	};
	($lt: lifetime) => {
		&$lt str
	}
}

#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! str_t {
	() => {
		impl Into<String>
	};
	($lt: lifetime) => {
		impl Into<String> + $lt
	}
}

#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! str_get {
	($var:expr) => {
		$var
	};
}

#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! str_get {
	($var:expr) => {
		$var.into()
	};
}

#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! str_clone {
	($var:expr) => {
		$var
	};
}

#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! str_clone {
	($var:expr) => {
		$var.clone()
	};
}

#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! str_owned {
	($var:expr) => {
		&$var
	};
}

#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! str_owned {
	($var:expr) => {
		$var
	};
}

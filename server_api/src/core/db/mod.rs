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
pub use self::sqlite::{exec, query, FormSqliteRowError, FromSqliteRow};

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

#[cfg(test)]
mod test
{
	use uuid::Uuid;

	use super::*;
	use crate::core::get_time;
	use crate::take_or_err;

	#[derive(Debug)]
	pub struct TestData
	{
		id: String,
		name: String,
		time: u128,
	}

	#[cfg(feature = "mysql")]
	impl mysql_async::prelude::FromRow for TestData
	{
		fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
		where
			Self: Sized,
		{
			Ok(TestData {
				id: take_or_err!(row, 0, String),
				name: take_or_err!(row, 1, String),
				time: take_or_err!(row, 2, u128),
			})
		}
	}

	#[cfg(feature = "sqlite")]
	impl crate::core::db::FromSqliteRow for TestData
	{
		fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::core::db::FormSqliteRowError>
		where
			Self: Sized,
		{
			//time needs to parse from string to the value
			let time: String = take_or_err!(row, 2);
			let time: u128 = time.parse().map_err(|e| {
				crate::core::db::FormSqliteRowError {
					msg: format!("err in db fetch: {:?}", e),
				}
			})?;

			Ok(TestData {
				id: take_or_err!(row, 0),
				name: take_or_err!(row, 1),
				time,
			})
		}
	}

	/**
		# Test the db
		This test should run for both sqlite and mysql

	*/
	#[tokio::test]
	async fn test_db_insert_and_fetch()
	{
		dotenv::dotenv().ok();

		init_db().await;

		//language=SQL
		let sql = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

		let id = Uuid::new_v4().to_string();
		let name = "hello".to_string();
		let time = get_time().unwrap();

		exec(sql, set_params!(id.clone(), name, time.to_string()))
			.await
			.unwrap();

		//fetch the new test data
		//language=SQL
		let sql = "SELECT * FROM test WHERE id = ?";

		let test_data = query::<TestData, _>(sql, set_params!(id.clone()))
			.await
			.unwrap();

		println!("out: {:?}", test_data);

		assert_eq!(test_data.len(), 1);
		assert_eq!(test_data[0].id, id);
	}
}

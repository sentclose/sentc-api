use std::error::Error;

use tokio::sync::OnceCell;

use crate::core::api_res::{ApiErrorCodes, HttpErr};

#[cfg(feature = "mysql")]
mod mariadb;
#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "mysql")]
pub use self::mariadb::{bulk_insert, exec, exec_transaction, query, query_first, TransactionData};
#[cfg(feature = "sqlite")]
pub use self::sqlite::{bulk_insert, exec, exec_transaction, query, query_first, FormSqliteRowError, FromSqliteRow, TransactionData};

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
	HttpErr::new(
		422,
		ApiErrorCodes::DbQuery,
		"db error".to_owned(),
		Some(format!("db fetch err, {:?}", e)),
	)
}

fn db_exec_err<E: Error>(e: &E) -> HttpErr
{
	HttpErr::new(
		422,
		ApiErrorCodes::DbExecute,
		"db error".to_owned(),
		Some(format!("db execute err, {:?}", e)),
	)
}

fn db_bulk_insert_err<E: Error>(e: &E) -> HttpErr
{
	HttpErr::new(
		422,
		ApiErrorCodes::DbBulkInsert,
		"db error".to_owned(),
		Some(format!("db bulk insert err, {:?}", e)),
	)
}

fn db_tx_err<E: Error>(e: &E) -> HttpErr
{
	HttpErr::new(
		422,
		ApiErrorCodes::DbTx,
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
		_name: String,
		_time: u128,
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
				_name: take_or_err!(row, 1, String),
				_time: take_or_err!(row, 2, u128),
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
				_name: take_or_err!(row, 1),
				_time: time,
			})
		}
	}

	/**
		# Test the db
		This test should run for both sqlite and mysql
	*/
	async fn test_db_insert_and_fetch()
	{
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

		let test_data: Vec<TestData> = query(sql.to_string(), set_params!(id.clone()))
			.await
			.unwrap();

		println!("out: {:?}", test_data);

		assert_eq!(test_data.len(), 1);
		assert_eq!(test_data[0].id, id);

		//test query first
		let test_datum: Option<TestData> = query_first(sql.to_string(), set_params!(id.clone()))
			.await
			.unwrap();

		assert_eq!(test_datum.unwrap().id, id);

		//test without result
		let test_datum: Option<TestData> = query_first(sql.to_string(), set_params!(id.clone() + "123"))
			.await
			.unwrap();

		let not_found_datum = match test_datum {
			None => true,
			Some(_) => false,
		};

		assert_eq!(not_found_datum, true);
	}

	async fn test_db_insert_and_fetch_with_get_ins()
	{
		//two inserts
		//language=SQL
		let sql = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

		let id1 = Uuid::new_v4().to_string();
		let name1 = "hello1".to_string();
		let time1 = get_time().unwrap();

		exec(sql, set_params!(id1.clone(), name1, time1.to_string()))
			.await
			.unwrap();

		//language=SQL
		let sql = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

		let id2 = Uuid::new_v4().to_string();
		let name2 = "hello2".to_string();
		let time2 = get_time().unwrap();

		exec(sql, set_params!(id2.clone(), name2, time2.to_string()))
			.await
			.unwrap();

		let params = vec![id1.clone(), id2.clone()];

		let ins = get_in(&params);

		//language=SQLx
		let sql = format!("SELECT * FROM test WHERE id IN ({}) ORDER BY name", ins);

		let test_data: Vec<TestData> = query(sql, params).await.unwrap();

		println!("out get in: {:?}", test_data);

		assert_eq!(test_data.len(), 2);
		assert_eq!(test_data[0].id, id1);
		assert_eq!(test_data[1].id, id2);
	}

	async fn test_db_bulk_insert()
	{
		//do this extra because we need the ids later to check if this values are in the db
		let id1 = Uuid::new_v4().to_string();
		let id2 = Uuid::new_v4().to_string();
		let id3 = Uuid::new_v4().to_string();

		let t1 = TestData {
			id: id1.to_string(),
			_name: "hello".to_string(),
			_time: get_time().unwrap(),
		};

		let t2 = TestData {
			id: id2.to_string(),
			_name: "hello2".to_string(),
			_time: get_time().unwrap(),
		};

		let t3 = TestData {
			id: id3.to_string(),
			_name: "hello3".to_string(),
			_time: get_time().unwrap(),
		};

		bulk_insert(
			false,
			"test".to_string(),
			vec!["id".to_string(), "name".to_string(), "time".to_string()],
			vec![t1, t2, t3],
			|ob| set_params!(ob.id.to_string(), ob._name.to_string(), ob._time.to_string()),
		)
		.await
		.unwrap();

		//check if the values are in the db
		let params = vec![id1.clone(), id2.clone(), id3.clone()];

		let ins = get_in(&params);

		//language=SQLx
		let sql = format!("SELECT * FROM test WHERE id IN ({}) ORDER BY name", ins);

		let test_data: Vec<TestData> = query(sql, params).await.unwrap();

		println!("out bulk insert: {:?}", test_data);

		assert_eq!(test_data.len(), 3);
		assert_eq!(test_data[0].id, id1);
		assert_eq!(test_data[1].id, id2);
		assert_eq!(test_data[2].id, id3);
	}

	async fn test_tx_exec()
	{
		//language=SQL
		let sql = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

		let id1 = Uuid::new_v4().to_string();
		let name1 = "hello1".to_string();
		let time1 = get_time().unwrap();

		//language=SQL
		let sql2 = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

		let id2 = Uuid::new_v4().to_string();
		let name2 = "hello2".to_string();
		let time2 = get_time().unwrap();

		//language=SQL
		let sql3 = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

		let id3 = Uuid::new_v4().to_string();
		let name3 = "hello3".to_string();
		let time3 = get_time().unwrap();

		exec_transaction(vec![
			TransactionData {
				sql,
				params: set_params!(id1.clone(), name1, time1.to_string()),
			},
			TransactionData {
				sql: sql2,
				params: set_params!(id2.clone(), name2, time2.to_string()),
			},
			TransactionData {
				sql: sql3,
				params: set_params!(id3.clone(), name3, time3.to_string()),
			},
		])
		.await
		.unwrap();

		let params = vec![id1.clone(), id2.clone(), id3.clone()];

		let ins = get_in(&params);

		//language=SQLx
		let sql = format!("SELECT * FROM test WHERE id IN ({}) ORDER BY name", ins);

		let test_data: Vec<TestData> = query(sql, params).await.unwrap();

		println!("out get in: {:?}", test_data);

		assert_eq!(test_data.len(), 3);
		assert_eq!(test_data[0].id, id1);
		assert_eq!(test_data[1].id, id2);
		assert_eq!(test_data[2].id, id3);
	}

	#[tokio::test]
	async fn test_start()
	{
		//in one test here because rust still don't support before all tests functions
		dotenv::dotenv().ok();

		init_db().await;

		test_db_insert_and_fetch().await;
		test_db_insert_and_fetch_with_get_ins().await;
		test_db_bulk_insert().await;
		test_tx_exec().await;
	}
}

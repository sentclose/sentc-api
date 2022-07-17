use std::env;

use mysql_async::prelude::{FromRow, Queryable};
use mysql_async::{from_value, Params, Pool, Row};

use crate::core::api_err::{ApiErrorCodes, HttpErr};
use crate::core::db::{db_exec_err, db_query_err, MARIA_DB_COMM};

pub async fn create_db() -> Pool
{
	let user = env::var("DB_USER").unwrap();
	let pw = env::var("DB_PASS").unwrap();
	let mysql_host = env::var("DB_HOST").unwrap();
	let db = env::var("DB_NAME").unwrap();

	let pool = Pool::new(format!("mysql://{}:{}@{}/{}", user, pw, mysql_host, db).as_str());

	//test the connection
	let result = pool
		.get_conn()
		.await
		.unwrap()
		.query_first::<Row, _>("SELECT 1")
		.await
		.unwrap()
		.unwrap()
		.unwrap();

	let result: i32 = from_value(result[0].clone());

	assert_eq!(result, 1);

	println!("init mariadb");

	pool
}

pub async fn get_conn() -> Result<mysql_async::Conn, HttpErr>
{
	//get conn with a loop because for very much workload we getting an err -> try again
	let maria_db = MARIA_DB_COMM.get().unwrap();

	let mut i = 0; //say how much iteration should be done before giving up

	loop {
		if i > 10 {
			return Err(HttpErr::new(
				500,
				ApiErrorCodes::NoDbConnection,
				"No db connection",
				Some("No connection after 10 tries".to_string()),
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
pub async fn query<T, P>(sql: &str, params: P) -> Result<Vec<T>, HttpErr>
where
	T: FromRow + Send + 'static,
	P: Into<Params> + Send,
{
	let mut conn = get_conn().await?;

	conn.exec::<T, _, P>(sql, params)
		.await
		.map_err(|e| db_query_err(&e))
}

pub async fn exec<P>(sql: &str, params: P) -> Result<(), HttpErr>
where
	P: Into<Params> + Send,
{
	let mut conn = get_conn().await?;

	conn.exec_drop(sql, params)
		.await
		.map_err(|e| db_exec_err(&e))
}

#[macro_export]
macro_rules! take_or_err {
	($row:expr, $index:expr, $t:ident) => {
		match $row.take_opt::<$t, _>($index) {
			Some(value) => {
				match value {
					Ok(ir) => ir,
					Err(mysql_async::FromValueError(_value)) => {
						return Err(mysql_async::FromRowError($row));
					},
				}
			},
			None => return Err(mysql_async::FromRowError($row)),
		}
	};
}

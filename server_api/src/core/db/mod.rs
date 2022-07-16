use tokio::sync::OnceCell;

#[cfg(feature = "mysql")]
mod mariadb;
#[cfg(feature = "sqlite")]
mod sqlite;

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

/*
TODO
	- create global exec and query fn: pass a T value of the fetched obj, if insert use exec drop
	- create entities with FromRow for mariadb and a new trait where we get values from sqlite
	- check pass sqlite row obj from sync to async fn (via deadpool)
 */

mod core;
mod user;

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
		rusqlite::params![$($param),+]
	}};
}

pub async fn start()
{
	//load the env
	dotenv::dotenv().ok();

	core::db::init_db().await;
}

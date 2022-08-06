use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::api_res::{ApiErrorCodes, HttpErr};

pub mod api_res;
pub mod cache;
pub mod db;
pub mod email;
pub mod input_helper;
pub mod url_helper;

pub fn get_time() -> Result<u128, HttpErr>
{
	//get the current time in millisec like here:
	// https://stackoverflow.com/questions/26593387/how-can-i-get-the-current-time-in-milliseconds
	// and here: https://doc.rust-lang.org/std/time/constant.UNIX_EPOCH.html

	match SystemTime::now().duration_since(UNIX_EPOCH) {
		Ok(n) => Ok(n.as_millis()),
		Err(_e) => {
			Err(HttpErr::new(
				500,
				ApiErrorCodes::UnexpectedTime,
				"Time went backwards".to_owned(),
				None,
			))
		},
	}
}

pub fn get_time_in_sec() -> Result<u64, HttpErr>
{
	match SystemTime::now().duration_since(UNIX_EPOCH) {
		Ok(n) => Ok(n.as_secs()),
		Err(_e) => {
			Err(HttpErr::new(
				500,
				ApiErrorCodes::UnexpectedTime,
				"Time went backwards".to_owned(),
				None,
			))
		},
	}
}

use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::api_err::{ApiErrorCodes, HttpErr};

pub mod api_err;
pub mod db;
pub mod input_helper;

pub fn get_time() -> Result<u128, HttpErr>
{
	//get the current time in millisec like here:
	// https://stackoverflow.com/questions/26593387/how-can-i-get-the-current-time-in-milliseconds
	// and here: https://doc.rust-lang.org/std/time/constant.UNIX_EPOCH.html

	match SystemTime::now().duration_since(UNIX_EPOCH) {
		Ok(n) => Ok(n.as_millis()),
		Err(_e) => Err(HttpErr::new(500, ApiErrorCodes::UnexpectedTimeError, "Time went backwards", None)),
	}
}

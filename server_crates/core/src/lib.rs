#![allow(clippy::too_many_arguments, clippy::manual_map, clippy::tabs_in_doc_comments)]

use std::time::{SystemTime, UNIX_EPOCH};

pub mod cache;
pub mod db;
pub mod email;
pub mod error;
pub mod file;
pub mod input_helper;
pub mod res;
pub mod url_helper;

pub fn get_time() -> Result<u128, error::SentcCoreError>
{
	//get the current time in millisec like here:
	// https://stackoverflow.com/questions/26593387/how-can-i-get-the-current-time-in-milliseconds
	// and here: https://doc.rust-lang.org/std/time/constant.UNIX_EPOCH.html

	match SystemTime::now().duration_since(UNIX_EPOCH) {
		Ok(n) => Ok(n.as_millis()),
		Err(_e) => {
			Err(error::SentcCoreError::new_msg(
				500,
				error::CoreErrorCodes::UnexpectedTime,
				"Time went backwards",
			))
		},
	}
}

pub fn get_time_in_sec() -> Result<u64, error::SentcCoreError>
{
	match SystemTime::now().duration_since(UNIX_EPOCH) {
		Ok(n) => Ok(n.as_secs()),
		Err(_e) => {
			Err(error::SentcCoreError::new_msg(
				500,
				error::CoreErrorCodes::UnexpectedTime,
				"Time went backwards",
			))
		},
	}
}

#[cfg(feature = "derive_macro")]
#[allow(unused_imports)]
#[macro_use]
extern crate server_core_macros;

#[cfg(feature = "derive_macro")]
pub use server_core_macros::*;

use crate::error::SentcErrorConstructor;

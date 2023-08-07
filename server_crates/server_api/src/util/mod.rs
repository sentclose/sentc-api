use chrono::{Datelike, TimeZone, Utc};
use rustgram_server_util::res::AppRes;

pub mod api_res;

pub(crate) fn get_begin_of_month() -> AppRes<i64>
{
	let current_date = Utc::now();

	// Create a new DateTime representing the beginning of the current month
	let beginning_of_month = Utc.with_ymd_and_hms(current_date.year(), current_date.month(), 1, 0, 0, 0);

	Ok(beginning_of_month.unwrap().timestamp_millis())
}

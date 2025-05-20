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

/*
use std::future::Future;
use std::time::Duration;
use rand::Rng;
use rustgram_server_util::error::ServerCoreError;
use tokio::time::sleep;

pub(crate) async fn delete_with_retry<F, T, Fut>(mut f: F, max_retries: u32, initial_delay: u64) -> Result<T, ServerCoreError>
where
	F: FnMut() -> Fut,
	Fut: Future<Output = Result<T, ServerCoreError>>,
{
	let mut attempts = 0;
	let mut delay = initial_delay;

	loop {
		return match f().await {
			Ok(result) => Ok(result),
			Err(e) => {
				if let Some(msg) = &e.debug_msg {
					if msg.contains("Deadlock found") && attempts < max_retries {
						attempts += 1;
						// Add some randomization to help prevent deadlocks
						let jitter = rand::thread_rng().gen_range(0..50);
						sleep(Duration::from_millis(delay + jitter)).await;
						delay *= 2; // exponential backoff
						continue;
					}
				}
				Err(e)
			},
		};
	}
}


async fn retry_on_deadlock<F, T, Fut>(mut f: F, max_retries: u32) -> Result<T, ServerCoreError>
where
	F: FnMut() -> Fut,
	Fut: Future<Output = Result<T, ServerCoreError>>,
{
	let mut attempts = 0;
	loop {
		return match f().await {
			Ok(result) => Ok(result),
			Err(e) => {
				if let Some(msg) = &e.debug_msg {
					if msg.contains("Deadlock found") && attempts < max_retries {
						attempts += 1;
						sleep(Duration::from_millis(50 * attempts as u64)).await;
						continue;
					}
				}

				Err(e)
			},
		};
	}
}
 */

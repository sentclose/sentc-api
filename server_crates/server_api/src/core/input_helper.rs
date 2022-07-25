use bytes::BytesMut;
use futures::StreamExt;
use rustgram::Request;
use serde::{de, Serialize};
use serde_json::{from_slice, to_string};

use crate::core::api_res::{ApiErrorCodes, HttpErr};

const MAX_SIZE_JSON: usize = 262_144; // max payload size is 256k

pub async fn get_raw_body(req: &mut Request) -> Result<BytesMut, HttpErr>
{
	//read the json to memory
	let req_body = req.body_mut();
	let mut body = BytesMut::new();

	while let Some(bytes) = req_body.next().await {
		match bytes {
			Ok(chunk) => {
				if (body.len() + chunk.len()) > MAX_SIZE_JSON {
					return Err(HttpErr::new(
						413,
						ApiErrorCodes::InputTooBig,
						"Input was too big to handle",
						None,
					));
				}

				body.extend_from_slice(&chunk);
			},
			Err(_) => {
				continue;
			},
		}
	}

	Ok(body)
}

pub fn json_to_string<T>(value: &T) -> Result<String, HttpErr>
where
	T: ?Sized + Serialize,
{
	match to_string(value) {
		Ok(o) => Ok(o),
		Err(e) => {
			#[cfg(debug_assertions)]
			let debug = Some(format!("json parse err: {:?}", e));

			#[cfg(not(debug_assertions))]
			let debug = None;

			Err(HttpErr::new(
				422,
				ApiErrorCodes::JsonToString,
				"Cannot convert json to string",
				debug,
			))
		},
	}
}

pub fn bytes_to_json<'a, T>(v: &'a [u8]) -> Result<T, HttpErr>
where
	T: de::Deserialize<'a>,
{
	match from_slice::<T>(v) {
		Ok(o) => Ok(o),
		Err(e) => {
			#[cfg(debug_assertions)]
			let debug = Some(format!("wrong json format: {:?}", e));

			#[cfg(not(debug_assertions))]
			let debug = None;

			Err(HttpErr::new(422, ApiErrorCodes::JsonParse, "wrong input", debug))
		},
	}
}

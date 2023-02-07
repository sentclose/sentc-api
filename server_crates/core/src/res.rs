use hyper::StatusCode;
use rustgram::service::IntoResponse;
use rustgram::Response;
use serde::Serialize;

use crate::error::SentcCoreError;
use crate::get_time;
use crate::input_helper::json_to_string;

//__________________________________________________________________________________________________
//two different output for str msg and string msg but for the client is is always the same

#[derive(Serialize)]
pub struct ServerOutputStr<T>
{
	pub status: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub err_msg: Option<&'static str>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub err_code: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub result: Option<T>,
}

#[derive(Serialize)]
pub struct ServerOutput<T>
{
	pub status: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub err_msg: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub err_code: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub result: Option<T>,
}

//__________________________________________________________________________________________________

impl IntoResponse<Response> for SentcCoreError
{
	fn into_response(self) -> Response
	{
		let status = match StatusCode::from_u16(self.http_status_code) {
			Ok(s) => s,
			Err(_e) => StatusCode::BAD_REQUEST,
		};

		//msg for the developer only
		//for std out to get logged with 3rd party service.
		if let Some(m) = self.debug_msg {
			let time = get_time().unwrap_or(0);
			println!("Http Error at time: {} Error: {:?}", time, m);
		}

		let body = if let Some(m) = self.msg_owned {
			json_to_string(&ServerOutput::<String> {
				status: false,
				result: None,
				err_msg: Some(m),
				err_code: Some(self.error_code),
			})
			.unwrap()
		} else {
			json_to_string(&ServerOutputStr::<String> {
				status: false,
				result: None,
				err_msg: Some(self.msg),
				err_code: Some(self.error_code),
			})
			.unwrap()
		};

		hyper::Response::builder()
			.status(status)
			.header("Content-Type", "application/json")
			.header("Access-Control-Allow-Origin", "*")
			.body(hyper::Body::from(body))
			.unwrap()
	}
}

//__________________________________________________________________________________________________

/**
Creates a json response with the json header

Creates a string from the obj
 */
pub struct JsonRes<T: Serialize>(pub T);

impl<T: Serialize> IntoResponse<Response> for JsonRes<T>
{
	fn into_response(self) -> Response
	{
		let out = ServerOutput {
			status: true,
			err_msg: None,
			err_code: None,
			result: Some(self.0),
		};

		let string = match json_to_string(&out) {
			Ok(s) => s,
			Err(e) => return Into::<SentcCoreError>::into(e).into_response(),
		};

		hyper::Response::builder()
			.header("Content-Type", "application/json")
			.header("Access-Control-Allow-Origin", "*")
			.body(string.into())
			.unwrap()
	}
}

pub type AppRes<T> = Result<T, SentcCoreError>;

pub type JRes<T> = Result<JsonRes<T>, SentcCoreError>;

pub fn echo<T: Serialize>(obj: T) -> JRes<T>
{
	Ok(JsonRes(obj))
}

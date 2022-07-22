use std::error::Error;

use hyper::StatusCode;
use rustgram::{GramHttpErr, Response};

#[derive(Debug)]
pub enum ApiErrorCodes
{
	JsonToString,
	JsonParse,

	InputTooBig,

	UnexpectedTimeError,

	NoDbConnection,
	DbQuery,
	DbExecute,
	DbBulkInsert,

	JwtValidationFailed,

	UserNotFound,
}

impl ApiErrorCodes
{
	pub fn get_int_code(&self) -> u32
	{
		match self {
			ApiErrorCodes::JsonToString => 10,
			ApiErrorCodes::JsonParse => 11,
			ApiErrorCodes::InputTooBig => 12,
			ApiErrorCodes::UnexpectedTimeError => 12,
			ApiErrorCodes::NoDbConnection => 20,
			ApiErrorCodes::DbQuery => 21,
			ApiErrorCodes::DbExecute => 22,
			ApiErrorCodes::DbBulkInsert => 23,
			ApiErrorCodes::JwtValidationFailed => 30,
			ApiErrorCodes::UserNotFound => 100,
		}
	}
}

#[derive(Debug)]
pub struct HttpErr
{
	http_status_code: u16,
	api_error_code: ApiErrorCodes,
	msg: &'static str,
	debug_msg: Option<String>,
}

impl HttpErr
{
	pub fn new(http_status_code: u16, api_error_code: ApiErrorCodes, msg: &'static str, debug_msg: Option<String>) -> Self
	{
		Self {
			http_status_code,
			api_error_code,
			msg,
			debug_msg,
		}
	}
}

impl GramHttpErr<Response> for HttpErr
{
	fn get_res(&self) -> Response
	{
		let status = match StatusCode::from_u16(self.http_status_code) {
			Ok(s) => s,
			Err(_e) => StatusCode::BAD_REQUEST,
		};

		//the msg for the end user
		let msg = format!(
			"{{\"status\": {}, \"error_message\": \"{}\"}}",
			self.api_error_code.get_int_code(),
			self.msg
		);

		//msg for the developer only
		//this could later be logged
		if self.debug_msg.is_some() {
			//TODO handle debug msg
			println!("Http Error: {:?}", self.debug_msg);
		}

		hyper::Response::builder()
			.status(status)
			.header("Content-Type", "application/json")
			.body(hyper::Body::from(msg))
			.unwrap()
	}
}

pub fn json_to_string_err<E: Error>(e: E) -> HttpErr
{
	HttpErr::new(
		422,
		ApiErrorCodes::JsonToString,
		"Err in json",
		Some(format!("err in json to string: {:?}", e)),
	)
}

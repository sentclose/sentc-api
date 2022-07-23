use std::error::Error;

use hyper::StatusCode;
use rustgram::service::HttpResult;
use rustgram::{GramHttpErr, Response};
use serde::Serialize;

use crate::core::input_helper::json_to_string;

#[derive(Debug)]
pub enum ApiErrorCodes
{
	JsonToString,
	JsonParse,

	InputTooBig,

	UnexpectedTime,

	NoDbConnection,
	DbQuery,
	DbExecute,
	DbBulkInsert,

	JwtNotFound,
	JwtWrongFormat,
	JwtValidation,
	JwtCreation,
	JwtKeyCreation,
	JwtKeyNotFound,

	NoParameter,

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
			ApiErrorCodes::UnexpectedTime => 12,
			ApiErrorCodes::NoDbConnection => 20,
			ApiErrorCodes::DbQuery => 21,
			ApiErrorCodes::DbExecute => 22,
			ApiErrorCodes::DbBulkInsert => 23,
			ApiErrorCodes::JwtValidation => 30,
			ApiErrorCodes::JwtNotFound => 31,
			ApiErrorCodes::JwtWrongFormat => 32,
			ApiErrorCodes::JwtCreation => 33,
			ApiErrorCodes::JwtKeyCreation => 34,
			ApiErrorCodes::JwtKeyNotFound => 35,
			ApiErrorCodes::NoParameter => 40,
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

pub type JRes<T> = Result<JsonRes<T>, HttpErr>;

/**
Creates a json response with the json header

Creates a string from the obj
*/
pub struct JsonRes<T: ?Sized + Serialize>(pub T);

impl<T: ?Sized + Serialize> HttpResult<Response> for JsonRes<T>
{
	fn get_res(&self) -> Response
	{
		let string = match json_to_string(&self.0) {
			Ok(s) => s,
			Err(e) => return e.get_res(),
		};

		hyper::Response::builder()
			.header("Content-Type", "application/json")
			.body(string.into())
			.unwrap()
	}
}

pub fn echo<T: Serialize>(obj: T) -> JRes<T>
{
	Ok(JsonRes(obj))
}
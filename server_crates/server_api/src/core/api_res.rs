use hyper::StatusCode;
use rustgram::service::HttpResult;
use rustgram::{GramHttpErr, Response};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::ServerOutput;
use serde::Serialize;

use crate::core::input_helper::json_to_string;

#[derive(Debug)]
pub enum ApiErrorCodes
{
	PageNotFound,

	JsonToString,
	JsonParse,

	InputTooBig,

	UnexpectedTime,

	NoDbConnection,
	DbQuery,
	DbExecute,
	DbBulkInsert,
	DbTx,

	JwtNotFound,
	JwtWrongFormat,
	JwtValidation,
	JwtCreation,
	JwtKeyCreation,
	JwtKeyNotFound,

	NoParameter,

	UserNotFound,
	UserExists,
	Login,
	WrongJwtAction,

	AuthKeyFormat,

	SaltError,

	AppTokenNotFound,
	AppTokenWrongFormat,
	AppNotFound,

	GroupUserNotFound,
	GroupUserRank,
	GroupUserExists,
	GroupNoKeys,
}

impl ApiErrorCodes
{
	pub fn get_int_code(&self) -> u32
	{
		match self {
			ApiErrorCodes::PageNotFound => 404,
			ApiErrorCodes::JsonToString => 10,
			ApiErrorCodes::JsonParse => 11,
			ApiErrorCodes::InputTooBig => 12,
			ApiErrorCodes::UnexpectedTime => 12,
			ApiErrorCodes::NoDbConnection => 20,
			ApiErrorCodes::DbQuery => 21,
			ApiErrorCodes::DbExecute => 22,
			ApiErrorCodes::DbBulkInsert => 23,
			ApiErrorCodes::DbTx => 24,
			ApiErrorCodes::JwtValidation => 30,
			ApiErrorCodes::JwtNotFound => 31,
			ApiErrorCodes::JwtWrongFormat => 32,
			ApiErrorCodes::JwtCreation => 33,
			ApiErrorCodes::JwtKeyCreation => 34,
			ApiErrorCodes::JwtKeyNotFound => 35,
			ApiErrorCodes::NoParameter => 40,
			ApiErrorCodes::UserNotFound => 100,
			ApiErrorCodes::UserExists => 101,
			ApiErrorCodes::SaltError => 110,
			ApiErrorCodes::AuthKeyFormat => 111,
			ApiErrorCodes::Login => 112,
			ApiErrorCodes::WrongJwtAction => 113,
			ApiErrorCodes::AppTokenNotFound => 200,
			ApiErrorCodes::AppTokenWrongFormat => 201,
			ApiErrorCodes::AppNotFound => 202,
			ApiErrorCodes::GroupUserNotFound => 300,
			ApiErrorCodes::GroupUserRank => 301,
			ApiErrorCodes::GroupUserExists => 302,
			ApiErrorCodes::GroupNoKeys => 303,
		}
	}
}

#[derive(Debug)]
pub struct HttpErr
{
	http_status_code: u16,
	api_error_code: ApiErrorCodes,
	msg: String,
	debug_msg: Option<String>,
}

impl HttpErr
{
	pub fn new(http_status_code: u16, api_error_code: ApiErrorCodes, msg: String, debug_msg: Option<String>) -> Self
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
	fn get_res(self) -> Response
	{
		let status = match StatusCode::from_u16(self.http_status_code) {
			Ok(s) => s,
			Err(_e) => StatusCode::BAD_REQUEST,
		};

		//msg for the developer only
		//this could later be logged
		if self.debug_msg.is_some() {
			//TODO handle debug msg
			println!("Http Error: {:?}", self.debug_msg);
		}

		let body = ServerOutput::<String> {
			status: false,
			result: None,
			err_msg: Some(self.msg),
			err_code: Some(self.api_error_code.get_int_code()),
		};
		//this should be right everytime
		let body = json_to_string(&body).unwrap();

		hyper::Response::builder()
			.status(status)
			.header("Content-Type", "application/json")
			.body(hyper::Body::from(body))
			.unwrap()
	}
}

pub type AppRes<T> = Result<T, HttpErr>;

pub type JRes<T> = Result<JsonRes<T>, HttpErr>;

/**
Creates a json response with the json header

Creates a string from the obj
*/
pub struct JsonRes<T: Serialize>(pub T);

impl<T: Serialize> HttpResult<Response> for JsonRes<T>
{
	fn get_res(self) -> Response
	{
		let out = ServerOutput {
			status: true,
			err_msg: None,
			err_code: None,
			result: Some(self.0),
		};

		let string = match json_to_string(&out) {
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

pub fn echo_success() -> JRes<ServerSuccessOutput>
{
	echo(ServerSuccessOutput("Success".to_string()))
}

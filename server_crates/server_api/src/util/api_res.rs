use hyper::StatusCode;
use rustgram::service::HttpResult;
use rustgram::{GramHttpErr, Response};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::ServerOutput;
use serde::Serialize;
use server_core::error::{CoreError, CoreErrorCodes};
use server_core::input_helper::json_to_string;

#[derive(Debug)]
pub enum ApiErrorCodes
{
	Core(CoreErrorCodes),

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

	EmailSend,
	EmailMessage,

	CustomerWrongAppToken,
	CustomerEmailValidate,
	CustomerNotFound,
	CustomerEmailTokenValid,
	CustomerEmailSyntax,

	UserNotFound,
	UserExists,
	Login,
	WrongJwtAction,
	AuthKeyFormat,
	SaltError,
	RefreshToken,

	AppTokenNotFound,
	AppTokenWrongFormat,
	AppNotFound,
	AppAction,

	GroupUserNotFound,
	GroupUserRank,
	GroupUserExists,
	GroupNoKeys,
	GroupKeyNotFound,

	GroupTooManyKeys,
	GroupKeySession,
	GroupInviteNotFound,
	GroupOnlyOneAdmin,
	GroupJoinReqNotFound,
	GroupAccess,
	GroupKeyRotationKeysNotFound,
	GroupKeyRotationThread,
	GroupKeyRotationUserEncrypt,
	GroupUserRankUpdate,
	GroupUserKick,
	GroupUserKickRank,

	KeyNotFound,
}

impl From<CoreErrorCodes> for ApiErrorCodes
{
	fn from(e: CoreErrorCodes) -> Self
	{
		Self::Core(e)
	}
}

impl ApiErrorCodes
{
	pub fn get_int_code(&self) -> u32
	{
		match self {
			ApiErrorCodes::Core(core) => {
				match core {
					CoreErrorCodes::JsonToString => 10,
					CoreErrorCodes::JsonParse => 11,
					CoreErrorCodes::InputTooBig => 12,
					CoreErrorCodes::UnexpectedTime => 13,

					CoreErrorCodes::NoDbConnection => 20,
					CoreErrorCodes::DbQuery => 21,
					CoreErrorCodes::DbExecute => 22,
					CoreErrorCodes::DbBulkInsert => 23,
					CoreErrorCodes::DbTx => 24,

					CoreErrorCodes::NoParameter => 40,

					CoreErrorCodes::EmailSend => 50,
					CoreErrorCodes::EmailMessage => 51,
				}
			},

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

			ApiErrorCodes::EmailSend => 50,
			ApiErrorCodes::EmailMessage => 51,

			ApiErrorCodes::CustomerWrongAppToken => 60,
			ApiErrorCodes::CustomerEmailValidate => 61,
			ApiErrorCodes::CustomerNotFound => 62,
			ApiErrorCodes::CustomerEmailTokenValid => 63,
			ApiErrorCodes::CustomerEmailSyntax => 64,

			ApiErrorCodes::UserNotFound => 100,
			ApiErrorCodes::UserExists => 101,
			ApiErrorCodes::SaltError => 110,
			ApiErrorCodes::AuthKeyFormat => 111,
			ApiErrorCodes::Login => 112,
			ApiErrorCodes::WrongJwtAction => 113,
			ApiErrorCodes::RefreshToken => 114,

			ApiErrorCodes::AppTokenNotFound => 200,
			ApiErrorCodes::AppTokenWrongFormat => 201,
			ApiErrorCodes::AppNotFound => 202,
			ApiErrorCodes::AppAction => 203,

			ApiErrorCodes::GroupUserNotFound => 300,
			ApiErrorCodes::GroupUserRank => 301,
			ApiErrorCodes::GroupUserExists => 302,
			ApiErrorCodes::GroupNoKeys => 303,
			ApiErrorCodes::GroupKeyNotFound => 304,

			ApiErrorCodes::GroupTooManyKeys => 305,
			ApiErrorCodes::GroupKeySession => 306,
			ApiErrorCodes::GroupInviteNotFound => 307,
			ApiErrorCodes::GroupOnlyOneAdmin => 308,
			ApiErrorCodes::GroupJoinReqNotFound => 309,
			ApiErrorCodes::GroupAccess => 310,
			ApiErrorCodes::GroupKeyRotationKeysNotFound => 311,
			ApiErrorCodes::GroupKeyRotationThread => 312,
			ApiErrorCodes::GroupKeyRotationUserEncrypt => 313,
			ApiErrorCodes::GroupUserRankUpdate => 314,
			ApiErrorCodes::GroupUserKick => 315,
			ApiErrorCodes::GroupUserKickRank => 316,

			ApiErrorCodes::KeyNotFound => 400,
		}
	}
}

#[derive(Debug)]
pub struct HttpErr
{
	http_status_code: u16,
	pub api_error_code: ApiErrorCodes,
	pub msg: String,
	debug_msg: Option<String>,
}

impl From<CoreError> for HttpErr
{
	fn from(e: CoreError) -> Self
	{
		Self {
			http_status_code: e.http_status_code,
			api_error_code: e.error_code.into(),
			msg: e.msg,
			debug_msg: e.debug_msg,
		}
	}
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
			.header("Access-Control-Allow-Origin", "*")
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
			Err(e) => return Into::<HttpErr>::into(e).get_res(),
		};

		hyper::Response::builder()
			.header("Content-Type", "application/json")
			.header("Access-Control-Allow-Origin", "*")
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

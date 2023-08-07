#![allow(clippy::tabs_in_doc_comments, clippy::from_over_into)]

use hyper::Body;
use rustgram::{r, Request, Response, Router};
use rustgram_server_util::error::{ServerErrorCodes, ServerErrorConstructor};

pub mod customer_app;
pub mod file;
pub mod group;
pub mod middleware;
pub mod user;
pub mod util;

pub const SENTC_ROOT_APP: &str = "sentc_int";

pub async fn start()
{
	//load the env
	dotenv::from_filename("sentc.env").ok();

	rustgram_server_util::db::init_db().await;
	rustgram_server_util::cache::init_cache().await;
	rustgram_server_util::file::init_storage().await;

	encrypted_at_rest_root::init_crypto().await;
}

/**
Entrypoint for every app

Use everytime, in standalone and other modes to get the router.

The other crates can use this router
 */
pub fn rest_routes() -> Router
{
	let mut router = Router::new(not_found_handler);

	router.get("/", r(index_handler));

	//cors route
	router.options("/*all", r(cors_handler));

	router
}

pub async fn not_found_handler(_req: Request) -> rustgram_server_util::res::JRes<String>
{
	Err(rustgram_server_util::error::ServerCoreError::new_msg(
		404,
		ApiErrorCodes::PageNotFound,
		"Not found",
	))
}

pub async fn index_handler(_req: Request) -> Response
{
	hyper::Response::builder()
		.status(hyper::StatusCode::MOVED_PERMANENTLY)
		.header("Location", "/dashboard")
		.header("Access-Control-Allow-Origin", "*")
		.body(Body::from(""))
		.unwrap()
}

pub async fn cors_handler(_req: Request) -> Response
{
	hyper::Response::builder()
		.header("Content-Length", "0")
		.header(
			"Access-Control-Allow-Methods",
			"GET, POST, PUT, DELETE, OPTIONS, PATCH",
		)
		.header("Access-Control-Max-Age", "86400")
		.header("Access-Control-Allow-Origin", "*")
		.header("Access-Control-Allow-Credentials", "true")
		.header(
			"Access-Control-Allow-Headers",
			"x-sentc-app-token, x-sentc-group-access-id, Content-Type, Accept, Origin, Authorization",
		)
		.body(Body::from(""))
		.unwrap()
}

#[derive(Debug)]
pub enum ApiErrorCodes
{
	PageNotFound,

	JwtNotFound,
	JwtWrongFormat,
	JwtValidation,
	JwtCreation,
	JwtKeyCreation,
	JwtKeyNotFound,

	UserNotFound,

	AppTokenNotFound,
	AppTokenWrongFormat,
	AppNotFound,
	AppAction,

	GroupAccess,

	CaptchaCreate,
	CaptchaNotFound,
	CaptchaTooOld,
	CaptchaWrong,
}

impl ServerErrorCodes for ApiErrorCodes
{
	fn get_int_code(&self) -> u32
	{
		match self {
			ApiErrorCodes::PageNotFound => 404,

			ApiErrorCodes::JwtValidation => 30,
			ApiErrorCodes::JwtNotFound => 31,
			ApiErrorCodes::JwtWrongFormat => 32,
			ApiErrorCodes::JwtCreation => 33,
			ApiErrorCodes::JwtKeyCreation => 34,
			ApiErrorCodes::JwtKeyNotFound => 35,

			ApiErrorCodes::UserNotFound => 100,

			ApiErrorCodes::AppTokenNotFound => 200,
			ApiErrorCodes::AppTokenWrongFormat => 201,
			ApiErrorCodes::AppNotFound => 202,
			ApiErrorCodes::AppAction => 203,

			ApiErrorCodes::GroupAccess => 310,

			ApiErrorCodes::CaptchaCreate => 600,
			ApiErrorCodes::CaptchaNotFound => 601,
			ApiErrorCodes::CaptchaTooOld => 602,
			ApiErrorCodes::CaptchaWrong => 603,
		}
	}
}

#![allow(clippy::tabs_in_doc_comments, clippy::from_over_into)]

use rustgram::Router;
use rustgram_server_util::error::ServerErrorCodes;

pub mod customer;
pub mod customer_app;
mod email;
pub mod mw;
mod routes;

pub async fn start()
{
	email::init_email_checker().await;
	#[cfg(feature = "send_mail")]
	email::send_mail::init_email_register().await;
}

pub fn customer_routes(router: &mut Router)
{
	routes::routes(router)
}

#[derive(Debug)]
pub enum ApiErrorCodes
{
	CustomerWrongAppToken,
	CustomerEmailValidate,
	CustomerNotFound,
	CustomerEmailTokenValid,
	CustomerEmailSyntax,
	CustomerDisable,

	AppTokenNotFound,
	AppTokenWrongFormat,
	AppNotFound,
	AppAction,

	GroupAccess,
	GroupUserRank,
}

impl ServerErrorCodes for ApiErrorCodes
{
	fn get_int_code(&self) -> u32
	{
		match self {
			ApiErrorCodes::CustomerWrongAppToken => 60,
			ApiErrorCodes::CustomerEmailValidate => 61,
			ApiErrorCodes::CustomerNotFound => 62,
			ApiErrorCodes::CustomerEmailTokenValid => 63,
			ApiErrorCodes::CustomerEmailSyntax => 64,
			ApiErrorCodes::CustomerDisable => 65,

			ApiErrorCodes::AppTokenNotFound => 200,
			ApiErrorCodes::AppTokenWrongFormat => 201,
			ApiErrorCodes::AppNotFound => 202,
			ApiErrorCodes::AppAction => 203,

			ApiErrorCodes::GroupAccess => 310,
			ApiErrorCodes::GroupUserRank => 301,
		}
	}
}

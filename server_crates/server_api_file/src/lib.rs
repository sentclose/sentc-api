#![allow(clippy::tabs_in_doc_comments, clippy::from_over_into)]

use rustgram_server_util::error::ServerErrorCodes;

pub mod file_controller;
pub mod file_entities;
mod file_model;
pub mod file_service;
pub mod file_worker;
mod routes;

use rustgram::Router;

pub fn file_routes(router: &mut Router)
{
	routes::routes(router)
}

#[derive(Debug)]
pub enum ApiErrorCodes
{
	UnexpectedTime,

	FileSessionNotFound,
	FileSessionExpired,
	FileNotFound,
	FileUploadAllowed,
	FileAccess,
}

impl ServerErrorCodes for ApiErrorCodes
{
	fn get_int_code(&self) -> u32
	{
		match self {
			ApiErrorCodes::UnexpectedTime => 12,
			ApiErrorCodes::FileSessionNotFound => 510,
			ApiErrorCodes::FileSessionExpired => 511,
			ApiErrorCodes::FileNotFound => 512,
			ApiErrorCodes::FileUploadAllowed => 520,
			ApiErrorCodes::FileAccess => 521,
		}
	}
}

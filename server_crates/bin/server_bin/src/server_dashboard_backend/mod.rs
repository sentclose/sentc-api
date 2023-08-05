use std::env;
use std::ffi::OsStr;
use std::path::Path;

use hyper::header::ACCEPT_ENCODING;
use hyper::http::HeaderValue;
use rustgram::service::IntoResponse;
use rustgram::{r, Request, Response, Router};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::file::{FileHandler, LocalStorage};
use rustgram_server_util::url_helper::get_name_param_from_req;
use server_api::util::api_res::ApiErrorCodes;
use tokio::sync::OnceCell;

static LOCAL_FILE_HANDLER: OnceCell<Box<LocalStorage>> = OnceCell::const_new();

pub async fn start()
{
	let path = env::var("LOCAL_FRONTED_DIR").unwrap_or_else(|_| "dist".to_string());

	LOCAL_FILE_HANDLER
		.get_or_init(move || async { Box::new(LocalStorage::new(path)) })
		.await;
}

/**
A Route to load the static files
 */
pub fn routes(router: &mut Router)
{
	router.get("/dashboard", r(read_file));
	router.get("/dashboard/*path", r(read_file));
}

/**
Load the static file from the static dir.
 */
async fn read_file(req: Request) -> Response
{
	let mut file = match get_name_param_from_req(&req, "path") {
		Ok(f) => f,
		Err(_e) => "index.html",
	};

	if file.is_empty() || file == "/" {
		file = "index.html"
	}

	let mut file = file.to_owned();

	let ext = match Path::new(&file).extension() {
		Some(e) => {
			match OsStr::to_str(e) {
				Some(s) => s,
				None => return ServerCoreError::new_msg(404, ApiErrorCodes::PageNotFound, "Page not found").into_response(),
			}
		},
		None => {
			file += "/index.html";

			"html"
		},
	};

	let content_type = match ext {
		"html" => "text/html",
		"js" => "application/javascript",
		"wasm" => "application/wasm",
		"ico" => "image/x-icon",
		"png" => "image/png",
		"jpg" => "image/jpeg",
		"jpeg" => "image/jpeg",
		"svg" => "image/svg+xml",
		"woff2" => "font/woff2",
		_ => return ServerCoreError::new_msg(404, ApiErrorCodes::PageNotFound, "Page not found").into_response(),
	};

	let headers = req.headers().get(ACCEPT_ENCODING);

	let encoding = match (ext == "js" || ext == "wasm", headers) {
		(true, Some(h)) => {
			if let Ok(h) = std::str::from_utf8(h.as_bytes()) {
				if h.contains("br") {
					//use brotli
					file += ".br";
					Some("br")
				} else if h.contains("gzip") {
					//use the zip js
					file += ".gz";
					Some("gzip")
				} else {
					None
				}
			} else {
				None
			}
		},
		_ => None,
	};

	let handler = LOCAL_FILE_HANDLER.get().unwrap();

	match handler.get_part(&file, Some(content_type)).await {
		Ok(mut res) => {
			let res_headers = res.headers_mut();

			res_headers.insert("Cache-Control", HeaderValue::from_static("public, max-age=31536000"));
			if let Some(e) = encoding {
				res_headers.insert("Content-Encoding", HeaderValue::from_static(e));
			}

			res
		},
		Err(_e) => {
			//try index
			match handler.get_part("index.html", Some("html")).await {
				Ok(res) => res,
				Err(e) => Into::<ServerCoreError>::into(e).into_response(),
			}
		},
	}
}

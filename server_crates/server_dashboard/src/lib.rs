use std::env;
use std::ffi::OsStr;
use std::path::Path;

use hyper::http::HeaderValue;
use rustgram::service::IntoResponse;
use rustgram::{r, Request, Response, Router};
use server_api::util::api_res::{ApiErrorCodes, HttpErr};
use server_core::file;
use server_core::file::FileHandler;
use server_core::url_helper::get_name_param_from_req;
use tokio::sync::OnceCell;

static LOCAL_FILE_HANDLER: OnceCell<Box<dyn FileHandler>> = OnceCell::const_new();

pub async fn start()
{
	let path = env::var("LOCAL_FRONTED_DIR").unwrap();

	LOCAL_FILE_HANDLER
		.get_or_init(move || async { file::get_local_storage(path) })
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

	if file == "" || file == "/" {
		file = "index.html"
	}

	let mut file = file.to_owned();

	let ext = match Path::new(&file).extension() {
		Some(e) => {
			match OsStr::to_str(e) {
				Some(s) => s,
				None => return HttpErr::new(404, ApiErrorCodes::PageNotFound, "Page not found".to_string(), None).into_response(),
			}
		},
		None => {
			file = file + "/index.html";

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
		_ => return HttpErr::new(404, ApiErrorCodes::PageNotFound, "Page not found".to_string(), None).into_response(),
	};

	let handler = LOCAL_FILE_HANDLER.get().unwrap();

	match handler.get_part(&file, Some(content_type)).await {
		Ok(mut res) => {
			res.headers_mut()
				.insert("Cache-Control", HeaderValue::from_static("public, max-age=31536000"));

			res
		},
		Err(_e) => {
			//try index
			match handler.get_part("index.html", Some("html")).await {
				Ok(res) => res,
				Err(e) => Into::<HttpErr>::into(e).into_response(),
			}
		},
	}
}

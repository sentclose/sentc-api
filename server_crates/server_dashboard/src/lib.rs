use std::env;

use rustgram::service::IntoResponse;
use rustgram::{r, Request, Response, Router};
use server_api::util::api_res::HttpErr;
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
	router.get("/api/dashboard/*path", r(read_file));
}

/**
Load the static file from the static dir.
 */
async fn read_file(req: Request) -> Response
{
	let file = match get_name_param_from_req(&req, "path") {
		Ok(f) => f,
		Err(e) => return Into::<HttpErr>::into(e).into_response(),
	};

	let handler = LOCAL_FILE_HANDLER.get().unwrap();

	match handler.get_part(file).await {
		Ok(res) => res,
		Err(e) => Into::<HttpErr>::into(e).into_response(),
	}
}

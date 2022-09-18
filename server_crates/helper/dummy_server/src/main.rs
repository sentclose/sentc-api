mod test_external_file;

use rustgram::{r, Request, Router};

async fn not_found_handler(_req: Request) -> &'static str
{
	"404"
}

#[tokio::main]
async fn main()
{
	let mut router = Router::new(not_found_handler);

	router.get("/file_part/upload/:part_id", r(test_external_file::upload_part));
	router.post("/file_part/delete", r(test_external_file::delete));

	let addr = format!("{}:{}", "127.0.0.1", 3003).parse().unwrap();

	rustgram::start(router, addr).await;
}

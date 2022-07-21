use rustgram::{r, Request, Router};

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
async fn read_file(_req: Request) -> String
{
	//TODO

	format!("file")
}

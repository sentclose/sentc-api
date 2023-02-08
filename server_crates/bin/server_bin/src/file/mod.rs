mod routes;

use rustgram::Router;

pub fn file_routes(router: &mut Router)
{
	routes::routes(router)
}

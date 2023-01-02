use rustgram::Router;

mod routes;

pub fn routes(router: &mut Router)
{
	routes::routes(router)
}

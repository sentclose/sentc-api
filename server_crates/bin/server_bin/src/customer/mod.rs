use rustgram::Router;

mod routes;

pub fn customer_routes(router: &mut Router)
{
	routes::routes(router)
}

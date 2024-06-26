use std::future::Future;
use std::sync::Arc;

use rustgram::service::{IntoResponse, Service};
use rustgram::{Request, Response};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::url_helper::get_name_param_from_req;

use crate::customer_app::get_app_data_from_req;
use crate::user::jwt::get_user_in_app;

pub struct UserCheckForceMiddleware<S>
{
	inner: Arc<S>,
}

impl<S> Service<Request> for UserCheckForceMiddleware<S>
where
	S: Service<Request, Output = Response>,
{
	type Output = S::Output;

	fn call(&self, req: Request) -> impl Future<Output = Self::Output> + Send + 'static
	{
		let next = self.inner.clone();

		async move {
			match check_user_in_app(&req).await {
				Ok(_) => {},
				Err(e) => return e.into_response(),
			}

			next.call(req).await
		}
	}
}

async fn check_user_in_app(req: &Request) -> AppRes<()>
{
	let app = get_app_data_from_req(req)?;

	let user_id = get_name_param_from_req(req, "user_id")?;

	get_user_in_app(&app.app_data.app_id, user_id).await?;

	Ok(())
}

pub fn user_check_force_transform<S>(inner: S) -> UserCheckForceMiddleware<S>
{
	UserCheckForceMiddleware {
		inner: Arc::new(inner),
	}
}

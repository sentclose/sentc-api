use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use rustgram::service::{IntoResponse, Service};
use rustgram::{Request, Response};
use server_core::res::AppRes;
use server_core::url_helper::get_name_param_from_req;

use crate::customer_app::app_model::get_app_general;
use crate::sentc_user_jwt_service::get_jwt_data_from_param;

pub struct AppAccess<S>
{
	inner: Arc<S>,
}

impl<S> Service<Request> for AppAccess<S>
where
	S: Service<Request, Output = Response>,
{
	type Output = S::Output;
	type Future = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

	fn call(&self, mut req: Request) -> Self::Future
	{
		let next = self.inner.clone();

		Box::pin(async move {
			if let Err(e) = access_check(&mut req).await {
				return e.into_response();
			}

			next.call(req).await
		})
	}
}

pub fn app_access_transform<S>(inner: S) -> AppAccess<S>
{
	AppAccess {
		inner: Arc::new(inner),
	}
}

async fn access_check(req: &mut Request) -> AppRes<()>
{
	let user = get_jwt_data_from_param(req)?;
	let app_id = get_name_param_from_req(req, "app_id")?;

	let data = get_app_general(app_id, &user.id).await?;

	req.extensions_mut().insert(data);

	Ok(())
}

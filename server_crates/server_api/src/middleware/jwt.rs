use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use rustgram::service::Service;
use rustgram::{GramHttpErr, Request, Response};

use crate::core::api_err::{ApiErrorCodes, HttpErr};

pub struct JwtMiddleware<S>
{
	inner: Arc<S>,
	optional: bool,
}

impl<S> Service<Request> for JwtMiddleware<S>
where
	S: Service<Request, Output = Response>,
{
	type Output = S::Output;
	type Future = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

	fn call(&self, mut req: Request) -> Self::Future
	{
		let opt = self.optional;
		let next = self.inner.clone();

		Box::pin(async move {
			match jwt_check(&mut req, opt).await {
				Ok(_) => {},
				Err(e) => return e.get_res(),
			}

			next.call(req).await
		})
	}
}

pub fn jwt_transform<S>(inner: S) -> JwtMiddleware<S>
{
	JwtMiddleware {
		inner: Arc::new(inner),
		optional: false,
	}
}

// pub fn jwt_transform_opt<S>(inner: S) -> JwtMiddleware<S>
// {
// 	JwtMiddleware {
// 		inner: Arc::new(inner),
// 		optional: true,
// 	}
// }

async fn jwt_check(req: &mut Request, optional: bool) -> Result<(), HttpErr>
{
	//get and validate the jwt. then save it in the req param.
	//cache the jwt under with the jwt hash as key to save the validation process everytime. save false jwt too

	let check = true;

	if !check && !optional {
		//when valid jwt is required then err, otherwise just set the false jwt token into req params
		return Err(HttpErr::new(401, ApiErrorCodes::JwtValidationFailed, "No valid jwt", None));
	}

	Ok(())
}

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use rustgram::service::Service;
use rustgram::{GramHttpErr, Request, Response};

use crate::core::api_res::{ApiErrorCodes, HttpErr};
use crate::core::cache;
use crate::core::cache::{CacheVariant, LONG_TTL};
use crate::core::input_helper::{bytes_to_json, json_to_string};
use crate::customer_app::app_entities::AppData;
use crate::customer_app::app_model;
use crate::customer_app::app_util::hash_token_from_string_to_string;
use crate::util::APP_TOKEN_CACHE;

pub struct AppTokenMiddleware<S>
{
	inner: Arc<S>,
}

impl<S> Service<Request> for AppTokenMiddleware<S>
where
	S: Service<Request, Output = Response>,
{
	type Output = S::Output;
	type Future = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

	fn call(&self, mut req: Request) -> Self::Future
	{
		let next = self.inner.clone();

		Box::pin(async move {
			match token_check(&mut req).await {
				Ok(_) => {},
				Err(e) => return e.get_res(),
			}

			next.call(req).await
		})
	}
}

pub fn app_token_transform<S>(inner: S) -> AppTokenMiddleware<S>
{
	AppTokenMiddleware {
		inner: Arc::new(inner),
	}
}

async fn token_check(req: &mut Request) -> Result<(), HttpErr>
{
	let app_token = get_from_req(req)?;
	//hash the app token
	let hashed_token = hash_token_from_string_to_string(app_token.as_str())?;

	//load the app info from cache
	let key = APP_TOKEN_CACHE.to_string() + hashed_token.as_str();

	let entity = match cache::get(key.as_str()).await {
		Some(j) => bytes_to_json(j.as_bytes())?,
		None => {
			//load the info from the db
			let data = match app_model::get_app_data(hashed_token.as_str()).await {
				Ok(d) => d,
				Err(e) => {
					//save the wrong token in the cache
					cache::add(key, json_to_string(&CacheVariant::<AppData>::None)?, LONG_TTL).await;

					return Err(e);
				},
			};

			let data = CacheVariant::Some(data);

			//cache the info
			cache::add(key, json_to_string(&data)?, LONG_TTL).await;

			data
		},
	};

	let entity = match entity {
		CacheVariant::Some(d) => d,
		CacheVariant::None => {
			return Err(HttpErr::new(
				401,
				ApiErrorCodes::AppTokenNotFound,
				"No valid app token".to_owned(),
				None,
			))
		},
	};

	req.extensions_mut().insert(entity);

	Ok(())
}

fn get_from_req(req: &Request) -> Result<String, HttpErr>
{
	let headers = req.headers();
	let header = match headers.get("x-sentc-app-token") {
		Some(v) => v,
		None => {
			return Err(HttpErr::new(
				401,
				ApiErrorCodes::AppTokenNotFound,
				"No valid app token".to_owned(),
				None,
			))
		},
	};

	let app_token = std::str::from_utf8(header.as_bytes()).map_err(|_e| {
		HttpErr::new(
			401,
			ApiErrorCodes::AppTokenWrongFormat,
			"Wrong format".to_owned(),
			None,
		)
	})?;

	Ok(app_token.to_string())
}

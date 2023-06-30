use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use rustgram::service::{IntoResponse, Service};
use rustgram::{Request, Response};
use rustgram_server_util::cache;
use rustgram_server_util::cache::{CacheVariant, LONG_TTL};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::input_helper::{bytes_to_json, json_to_string};
use rustgram_server_util::res::AppRes;

use crate::customer_app::app_entities::AppData;
use crate::customer_app::app_model;
use crate::customer_app::app_util::hash_token_from_string_to_string;
use crate::util::api_res::ApiErrorCodes;
use crate::util::APP_TOKEN_CACHE;
use crate::SENTC_ROOT_APP;

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
				Err(e) => return e.into_response(),
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

/**
Middleware to check if the right sentc base app was used. Only used for internal routes
*/
pub struct AppTokenBaseAppMiddleware<S>
{
	inner: Arc<S>,
}

impl<S> Service<Request> for AppTokenBaseAppMiddleware<S>
where
	S: Service<Request, Output = Response>,
{
	type Output = S::Output;
	type Future = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

	fn call(&self, mut req: Request) -> Self::Future
	{
		let next = self.inner.clone();

		Box::pin(async move {
			match root_app_check(&mut req).await {
				Ok(_) => {},
				Err(e) => return e.into_response(),
			}

			next.call(req).await
		})
	}
}

pub fn app_token_base_app_transform<S>(inner: S) -> AppTokenBaseAppMiddleware<S>
{
	AppTokenBaseAppMiddleware {
		inner: Arc::new(inner),
	}
}

//__________________________________________________________________________________________________

async fn token_check(req: &mut Request) -> Result<(), ServerCoreError>
{
	let app_token = get_from_req(req)?;
	//hash the app token
	let hashed_token = hash_token_from_string_to_string(app_token.as_str())?;

	//load the app info from cache
	let key = APP_TOKEN_CACHE.to_string() + hashed_token.as_str();

	let entity = match cache::get(key.as_str()).await? {
		Some(j) => bytes_to_json(j.as_bytes())?,
		None => {
			//load the info from the db
			let data = match app_model::get_app_data(hashed_token).await {
				Ok(d) => d,
				Err(e) => {
					//save the wrong token in the cache
					cache::add(key, json_to_string(&CacheVariant::<AppData>::None)?, LONG_TTL).await?;

					return Err(e);
				},
			};

			let data = CacheVariant::Some(data);

			//cache the info
			cache::add(key, json_to_string(&data)?, LONG_TTL).await?;

			data
		},
	};

	let entity = match entity {
		CacheVariant::Some(d) => d,
		CacheVariant::None => {
			return Err(ServerCoreError::new_msg(
				401,
				ApiErrorCodes::AppTokenNotFound,
				"No valid app token",
			))
		},
	};

	req.extensions_mut().insert(entity);

	Ok(())
}

fn get_from_req(req: &Request) -> Result<String, ServerCoreError>
{
	let headers = req.headers();
	let header = match headers.get("x-sentc-app-token") {
		Some(v) => v,
		None => {
			return Err(ServerCoreError::new_msg(
				401,
				ApiErrorCodes::AppTokenNotFound,
				"No valid app token",
			))
		},
	};

	let app_token =
		std::str::from_utf8(header.as_bytes()).map_err(|_e| ServerCoreError::new_msg(401, ApiErrorCodes::AppTokenWrongFormat, "Wrong format"))?;

	Ok(app_token.to_string())
}

//__________________________________________________________________________________________________

async fn root_app_check(req: &mut Request) -> AppRes<()>
{
	//use the sentc base app as ref not a token

	//load the app info from cache
	let key = APP_TOKEN_CACHE.to_string() + SENTC_ROOT_APP;

	let entity = match cache::get(key.as_str()).await? {
		Some(j) => bytes_to_json(j.as_bytes())?,
		None => {
			//load the info from the db
			let data = match app_model::get_app_data_from_id(SENTC_ROOT_APP).await {
				Ok(d) => d,
				Err(e) => {
					//save the wrong token in the cache
					cache::add(key, json_to_string(&CacheVariant::<AppData>::None)?, LONG_TTL).await?;

					return Err(e);
				},
			};

			let data = CacheVariant::Some(data);

			//cache the info
			cache::add(key, json_to_string(&data)?, LONG_TTL).await?;

			data
		},
	};

	let entity = match entity {
		CacheVariant::Some(d) => d,
		CacheVariant::None => {
			return Err(ServerCoreError::new_msg(
				401,
				ApiErrorCodes::AppTokenNotFound,
				"No valid app token",
			))
		},
	};

	req.extensions_mut().insert(entity);

	Ok(())
}

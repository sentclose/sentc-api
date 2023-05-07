use std::collections::hash_map::DefaultHasher;
use std::env;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::Arc;

use hyper::header::AUTHORIZATION;
use rustgram::service::{IntoResponse, Service};
use rustgram::{Request, Response};
use rustgram_server_util::cache;
use rustgram_server_util::cache::{CacheVariant, DEFAULT_TTL};
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::input_helper::{bytes_to_json, json_to_string};
use sentc_crypto_common::AppId;

use crate::sentc_app_utils::get_app_data_from_req;
use crate::user::jwt::auth;
use crate::user::user_entities::UserJwtEntity;
use crate::util::api_res::ApiErrorCodes;
use crate::util::get_user_jwt_key;

const BEARER: &str = "Bearer ";

pub struct JwtMiddleware<S>
{
	inner: Arc<S>,
	optional: bool,
	check_exp: bool,
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
		let check_exp = self.check_exp;
		let next = self.inner.clone();

		Box::pin(async move {
			//get the app id. the app mw should run first everytime when using the jwt mw
			let app = match get_app_data_from_req(&req) {
				Ok(app) => app.app_data.app_id.clone(),
				Err(e) => return e.into_response(),
			};

			match jwt_check(&mut req, opt, check_exp, app).await {
				Ok(_) => {},
				Err(e) => return e.into_response(),
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
		check_exp: true,
	}
}

pub fn jwt_expire_transform<S>(inner: S) -> JwtMiddleware<S>
{
	JwtMiddleware {
		inner: Arc::new(inner),
		optional: false,
		check_exp: false,
	}
}

pub fn jwt_optional_transform<S>(inner: S) -> JwtMiddleware<S>
{
	JwtMiddleware {
		inner: Arc::new(inner),
		optional: true,
		check_exp: true,
	}
}

//__________________________________________________________________________________________________
//mw that uses the sentc internal app id for customer
pub struct JwtMiddlewareApp<S>
{
	inner: Arc<S>,
	sentc_app_id: AppId,
}

impl<S> Service<Request> for JwtMiddlewareApp<S>
where
	S: Service<Request, Output = Response>,
{
	type Output = S::Output;
	type Future = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

	fn call(&self, mut req: Request) -> Self::Future
	{
		let app_id = self.sentc_app_id.to_string();
		let next = self.inner.clone();

		Box::pin(async move {
			match jwt_check(&mut req, false, true, app_id).await {
				Ok(_) => {},
				Err(e) => return e.into_response(),
			}

			next.call(req).await
		})
	}
}

pub fn jwt_customer_app_transform<S>(inner: S) -> JwtMiddlewareApp<S>
{
	let sentc_app_id = env::var("SENTC_APP_ID").unwrap();

	JwtMiddlewareApp {
		inner: Arc::new(inner),
		sentc_app_id,
	}
}

//__________________________________________________________________________________________________

async fn jwt_check(req: &mut Request, optional: bool, check_exp: bool, app_id: AppId) -> Result<(), ServerCoreError>
{
	//get and validate the jwt. then save it in the req param.
	//cache the jwt under with the jwt hash as key to save the validation process everytime. save false jwt too

	let user = match get_jwt_from_req(req) {
		Err(e) => {
			if !optional {
				return Err(e);
			}
			None
		},
		Ok(jwt) => {
			match validate(app_id, jwt.as_str(), check_exp).await {
				Err(e) => {
					if !optional {
						return Err(e);
					}
					None
				},
				Ok(v) => Some(v),
			}
		},
	};

	//for non optional this is always Some
	req.extensions_mut().insert(user);

	Ok(())
}

fn get_jwt_from_req(req: &Request) -> Result<String, ServerCoreError>
{
	let headers = req.headers();
	let header = match headers.get(AUTHORIZATION) {
		Some(v) => v,
		None => {
			return Err(ServerCoreError::new_msg(
				401,
				ApiErrorCodes::JwtNotFound,
				"No valid jwt",
			))
		},
	};

	let auth_header =
		std::str::from_utf8(header.as_bytes()).map_err(|_e| ServerCoreError::new_msg(401, ApiErrorCodes::JwtWrongFormat, "Wrong format"))?;

	if !auth_header.starts_with(BEARER) {
		return Err(ServerCoreError::new_msg(
			401,
			ApiErrorCodes::JwtNotFound,
			"No valid jwt",
		));
	}

	Ok(auth_header.trim_start_matches(BEARER).to_string())
}

async fn validate(app_id: AppId, jwt: &str, check_exp: bool) -> Result<UserJwtEntity, ServerCoreError>
{
	//hash the jwt and check if it is in the cache

	//no need for crypto hasher
	let mut s = DefaultHasher::new();
	jwt.hash(&mut s);
	let cache_key = s.finish();
	let cache_key = cache_key.to_string();
	let cache_key = get_user_jwt_key(&app_id, &cache_key);

	let entity = match cache::get(cache_key.as_str()).await? {
		Some(j) => bytes_to_json(j.as_bytes())?,
		None => {
			//if not in the cache valid the jwt and cache it
			let (entity, exp) = match auth(app_id, jwt, check_exp).await {
				Ok(v) => v,
				Err(e) => {
					//save the wrong jwt in cache
					cache::add(
						cache_key,
						json_to_string(&CacheVariant::<UserJwtEntity>::None)?,
						DEFAULT_TTL,
					)
					.await?;

					return Err(e);
				},
			};

			let entity = CacheVariant::Some(entity);

			if check_exp {
				//only add the jwt to cache for exp able jwt's
				//ttl should end for this cache -1 sec before the actual token exp
				cache::add(cache_key, json_to_string(&entity)?, exp - 1).await?;
			}

			entity
		},
	};

	let entity = match entity {
		CacheVariant::Some(d) => d,
		CacheVariant::None => {
			return Err(ServerCoreError::new_msg(
				401,
				ApiErrorCodes::JwtNotFound,
				"No valid jwt",
			))
		},
	};

	Ok(entity)
}

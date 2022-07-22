use std::collections::hash_map::DefaultHasher;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::Arc;

use hyper::header::AUTHORIZATION;
use rustgram::service::Service;
use rustgram::{GramHttpErr, Request, Response};

use crate::core::api_err::{ApiErrorCodes, HttpErr};
use crate::core::cache;
use crate::core::cache::JWT_CACHE;
use crate::core::input_helper::{bytes_to_json, json_to_string};
use crate::core::jwt::auth;
use crate::user::user_entities::UserJwtEntity;

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
			match jwt_check(&mut req, opt, check_exp).await {
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
		check_exp: true,
	}
}

async fn jwt_check(req: &mut Request, optional: bool, check_exp: bool) -> Result<(), HttpErr>
{
	//get and validate the jwt. then save it in the req param.
	//cache the jwt under with the jwt hash as key to save the validation process everytime. save false jwt too

	let user = match get_jwt_from_req(&req) {
		Err(e) => {
			if !optional {
				return Err(e);
			}
			None
		},
		Ok(jwt) => {
			match validate(jwt.as_str(), check_exp).await {
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

	req.extensions_mut().insert(user);

	Ok(())
}

fn get_jwt_from_req(req: &Request) -> Result<String, HttpErr>
{
	let headers = req.headers();
	let header = match headers.get(AUTHORIZATION) {
		Some(v) => v,
		None => return Err(HttpErr::new(401, ApiErrorCodes::JwtNotFound, "No valid jwt", None)),
	};

	let auth_header = std::str::from_utf8(header.as_bytes()).map_err(|_e| HttpErr::new(401, ApiErrorCodes::JwtWrongFormat, "Wrong format", None))?;

	if !auth_header.starts_with(BEARER) {
		return Err(HttpErr::new(401, ApiErrorCodes::JwtNotFound, "No valid jwt", None));
	}

	Ok(auth_header.trim_start_matches(BEARER).to_string())
}

async fn validate(jwt: &str, check_exp: bool) -> Result<UserJwtEntity, HttpErr>
{
	//hash the jwt and check if it is in the cache

	//no need for crypto hasher
	let mut s = DefaultHasher::new();
	jwt.hash(&mut s);
	let cache_key = s.finish();
	let cache_key = cache_key.to_string();
	let cache_key = JWT_CACHE.to_string() + cache_key.as_str();

	let entity = match cache::get(cache_key.as_str()).await {
		Some(j) => bytes_to_json(j.as_bytes())?,
		None => {
			//if not in the cache valid the jwt and cache it
			let (entity, exp) = auth(jwt, check_exp).await?;

			if check_exp {
				//only add the jwt to cache for exp able jwt's
				//ttl should end for this cache -1 sec before the actual token exp
				cache::add(cache_key, json_to_string(&entity)?, exp - 1).await;
			}

			entity
		},
	};

	Ok(entity)
}

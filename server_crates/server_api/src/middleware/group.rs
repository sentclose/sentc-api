use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use rustgram::service::Service;
use rustgram::{GramHttpErr, Request, Response};

use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::core::cache;
use crate::core::cache::{CacheVariant, GROUP_DATA_CACHE, GROUP_USER_DATA_CACHE, LONG_TTL};
use crate::core::input_helper::{bytes_to_json, json_to_string};
use crate::core::url_helper::get_name_param_from_req;
use crate::group::group_entities::{InternalGroupData, InternalGroupDataComplete, InternalUserGroupData};
use crate::group::group_model;
use crate::user::jwt::get_jwt_data_from_param;

pub struct GroupMiddleware<S>
{
	inner: Arc<S>,
}

impl<S> Service<Request> for GroupMiddleware<S>
where
	S: Service<Request, Output = Response>,
{
	type Output = S::Output;
	type Future = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

	fn call(&self, mut req: Request) -> Self::Future
	{
		let next = self.inner.clone();

		Box::pin(async move {
			match get_group(&mut req).await {
				Ok(_) => {},
				Err(e) => return e.get_res(),
			}

			next.call(req).await
		})
	}
}

pub fn group_transform<S>(inner: S) -> GroupMiddleware<S>
{
	GroupMiddleware {
		inner: Arc::new(inner),
	}
}

async fn get_group(req: &mut Request) -> AppRes<()>
{
	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	let key_group = GROUP_DATA_CACHE.to_string() + user.sub.as_str() + "_" + group_id;
	let key_user = GROUP_USER_DATA_CACHE.to_string() + user.sub.as_str() + "_" + group_id + "_" + user.id.as_str();

	//use to different caches, one for the group, the other for the group user.
	//this is used because if a group gets deleted -> the cache of the user wont.

	let entity = match cache::get(key_group.as_str()).await {
		Some(j) => bytes_to_json(j.as_bytes())?,
		None => {
			let data = match group_model::get_group_data(user.sub.to_string(), group_id.to_string()).await {
				Ok(d) => d,
				Err(e) => {
					cache::add(
						key_group,
						json_to_string(&CacheVariant::<InternalGroupData>::None)?,
						LONG_TTL,
					)
					.await;

					return Err(e);
				},
			};

			let data = CacheVariant::Some(data);

			cache::add(key_group, json_to_string(&data)?, LONG_TTL).await;

			data
		},
	};

	let entity_group = match entity {
		CacheVariant::Some(d) => d,
		CacheVariant::None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::GroupAccess,
				"No access to this group".to_string(),
				None,
			))
		},
	};

	let entity = match cache::get(key_user.as_str()).await {
		Some(j) => bytes_to_json(j.as_bytes())?,
		None => {
			let data = match group_model::get_group_user_data(group_id.to_string(), user.id.to_string()).await {
				Ok(d) => d,
				Err(e) => {
					//cache wrong input too
					cache::add(
						key_user,
						json_to_string(&CacheVariant::<InternalUserGroupData>::None)?,
						LONG_TTL,
					)
					.await;

					return Err(e);
				},
			};

			let data = CacheVariant::Some(data);

			cache::add(key_user, json_to_string(&data)?, LONG_TTL).await;

			data
		},
	};

	let entity = match entity {
		CacheVariant::Some(d) => d,
		CacheVariant::None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::GroupAccess,
				"No access to this group".to_string(),
				None,
			))
		},
	};

	let group_data = InternalGroupDataComplete {
		group_data: entity_group,
		user_data: entity,
	};

	req.extensions_mut().insert(group_data);

	Ok(())
}

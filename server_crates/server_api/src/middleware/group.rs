use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use rustgram::service::{IntoResponse, Service};
use rustgram::{Request, Response};
use server_core::cache;
use server_core::cache::{CacheVariant, LONG_TTL, SHORT_TTL};
use server_core::input_helper::{bytes_to_json, json_to_string};
use server_core::url_helper::get_name_param_from_req;

use crate::customer_app::app_util::get_app_data_from_req;
use crate::group::group_entities::{InternalGroupData, InternalGroupDataComplete, InternalUserGroupData, InternalUserGroupDataFromParent};
use crate::group::group_model;
use crate::user::jwt::get_jwt_data_from_param;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::util::{get_group_cache_key, get_group_user_cache_key, get_group_user_parent_ref_key};

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
			match get_group_from_req(&mut req).await {
				Ok(_) => {},
				Err(e) => return e.into_response(),
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

async fn get_group_from_req(req: &mut Request) -> AppRes<()>
{
	let app = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;
	let group_id = get_name_param_from_req(&req, "group_id")?;

	//when access a group as group member not normal member
	let headers = req.headers();
	let group_as_member_id = match headers.get("x-sentc-group-access-id") {
		Some(v) => {
			let v = match std::str::from_utf8(v.as_bytes()) {
				Ok(v) => Some(v),
				Err(_e) => None,
			};

			v
		},
		None => None,
	};

	let group_data = get_group(
		app.app_data.app_id.as_str(),
		group_id,
		user.id.as_str(),
		group_as_member_id,
	)
	.await?;

	req.extensions_mut().insert(group_data);

	Ok(())
}

async fn get_group(app_id: &str, group_id: &str, user_id: &str, group_as_member_id: Option<&str>) -> AppRes<InternalGroupDataComplete>
{
	let key_group = get_group_cache_key(app_id, group_id);

	//use to different caches, one for the group, the other for the group user.
	//this is used because if a group gets deleted -> the cache of the user wont.

	let entity = match cache::get(key_group.as_str()).await {
		Some(j) => bytes_to_json(j.as_bytes())?,
		None => {
			let data = match group_model::get_internal_group_data(app_id.to_string(), group_id.to_string()).await {
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

	let (user_data, search_again) = get_group_user(app_id, group_id, user_id, group_as_member_id).await?;

	let user_data = if search_again {
		//when there was just a ref to a parent group for the user data -> get the parent group user data
		match user_data.get_values_from_parent {
			Some(id) => {
				let (result, _) = get_group_user(app_id, id.as_str(), user_id, group_as_member_id).await?;

				//create the user data from parent (rank in the parent group and jointed time)
				// and the user data of the child group
				let user_data = InternalUserGroupData {
					user_id: id.to_string(),
					real_user_id: user_id.to_string(),
					joined_time: user_data.joined_time,
					rank: result.rank,
					get_values_from_parent: Some(id),
					get_values_from_group_as_member: user_data.get_values_from_group_as_member,
				};

				user_data
			},
			None => user_data,
		}
	} else {
		user_data
	};

	let group_data = InternalGroupDataComplete {
		group_data: entity_group,
		user_data,
	};

	Ok(group_data)
}

/**
Example usage:

1. User is in group as direct member:
	- first check the cache if the user is in the cache:
	 - if not
		- go to the model and then the user should be there in this use case
		 - skip the check parent part
		 - cache the data
		 - return the data from the cache
	 - if in cache:
		 - just return the data from the cache
2. User is not a direct member but member of a parent group
	- first check the cache from this group if the user is in
	 - if not
		 - check the model if user is a direct member (in this use case not)
		 - search all parent groups via sql recursion if we found a parent group of this group where the user is member
		 - if no group found -> cache it for 5 min (because we don't know when the user joined any of the parent groups)
		 - if get the group values back
		 - build a cache from this values and store it with a ref on the real group data
			(this is important because when user joint a parent group later, then the cache from this child group is still wrong)
		 - return the data
	 - if in cache with a ref:
		 - return the data,
		 - in get_group fn we are searching for the real user data from the ref parent group again (mostly via cache), to see if the cache is still valid
*/
async fn get_group_user(app_id: &str, group_id: &str, user_id: &str, group_as_member_id: Option<&str>) -> AppRes<(InternalUserGroupData, bool)>
{
	//when the user wants to access the group by a group as member
	let check_user_id = match group_as_member_id {
		Some(v) => v,
		None => user_id,
	};

	let key_user = get_group_user_cache_key(app_id, group_id, check_user_id);

	let (entity, search_again) = match cache::get(key_user.as_str()).await {
		Some(j) => (bytes_to_json(j.as_bytes())?, true),
		None => {
			let data = match group_model::get_internal_group_user_data(group_id.to_string(), check_user_id.to_string()).await? {
				Some(mut d) => {
					d.get_values_from_group_as_member = match group_as_member_id {
						Some(v) => Some(v.to_string()),
						None => None,
					};

					//set always the real user id. just in case if user enters this group with another group
					d.real_user_id = user_id.to_string();

					d
				},
				None => {
					//check the parent ref to this group and user.
					let parent_ref = get_user_from_parent(group_id, check_user_id).await?;

					let d = InternalUserGroupData {
						user_id: parent_ref.get_values_from_parent.to_string(), //the the parent group id as user id when user comes from parent
						real_user_id: user_id.to_string(),
						joined_time: parent_ref.joined_time,
						rank: parent_ref.rank,
						//only set the ref to parent group here
						get_values_from_parent: Some(parent_ref.get_values_from_parent),
						get_values_from_group_as_member: match group_as_member_id {
							Some(v) => Some(v.to_string()),
							None => None,
						},
					};

					d
				},
			};

			let data = CacheVariant::Some(data);

			//cache the data everytime even if the user is not a direct member of the group,
			// if not direct member then work with reference to the parent group in get group fn
			cache::add(key_user, json_to_string(&data)?, LONG_TTL).await;

			//when user is direct member or we checked the parent group ref (with the real data)
			//we don't need to look up again if this data is still valid.
			(data, false)
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

	Ok((entity, search_again))
}

async fn get_user_from_parent(group_id: &str, user_id: &str) -> AppRes<InternalUserGroupDataFromParent>
{
	let key = get_group_user_parent_ref_key(group_id, user_id);

	let entity = match cache::get(key.as_str()).await {
		Some(v) => bytes_to_json(v.as_bytes())?,
		None => {
			//get the ref from the db
			let user_from_parent = match group_model::get_user_from_parent_groups(group_id.to_string(), user_id.to_string()).await? {
				Some(u) => u,
				None => {
					//cache wrong input too,
					// but only for 5 min because we don't know when the user joined any of the parent groups.
					cache::add(
						key,
						json_to_string(&CacheVariant::<InternalUserGroupDataFromParent>::None)?,
						SHORT_TTL,
					)
					.await;

					return Err(HttpErr::new(
						400,
						ApiErrorCodes::GroupAccess,
						"No access to this group".to_string(),
						None,
					));
				},
			};

			let data = CacheVariant::Some(user_from_parent);

			//when there is a ref -> cache it long
			cache::add(key, json_to_string(&data)?, LONG_TTL).await;

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

	Ok(entity)
}

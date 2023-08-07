use std::future::Future;

use rustgram_server_util::res::AppRes;
use sentc_crypto_common::{AppId, CustomerId, GroupId};

pub(crate) mod file_model;

pub const MAX_CHUNK_SIZE: usize = 5 * 1024 * 1024;
pub const MAX_SESSION_ALIVE_TIME: u128 = 24 * 60 * 60 * 1000;
pub const FILE_STATUS_AVAILABLE: i32 = 1;
pub const FILE_STATUS_TO_DELETE: i32 = 0;

pub const FILE_BELONGS_TO_TYPE_NONE: i32 = 0;
pub const FILE_BELONGS_TO_TYPE_GROUP: i32 = 1;
pub const FILE_BELONGS_TO_TYPE_USER: i32 = 2;

#[allow(clippy::needless_lifetimes)]
pub fn delete_file_for_customer<'a>(customer_id: impl Into<CustomerId> + 'a) -> impl Future<Output = AppRes<()>> + 'a
{
	file_model::delete_files_for_customer(customer_id)
}

#[allow(clippy::needless_lifetimes)]
pub fn delete_file_for_customer_group<'a>(group_id: impl Into<GroupId> + 'a) -> impl Future<Output = AppRes<()>> + 'a
{
	file_model::delete_files_for_customer_group(group_id)
}

#[allow(clippy::needless_lifetimes)]
pub fn delete_file_for_app<'a>(app_id: impl Into<AppId> + 'a) -> impl Future<Output = AppRes<()>> + 'a
{
	file_model::delete_files_for_app(app_id)
}

pub fn delete_file_for_group<'a>(
	app_id: impl Into<AppId> + 'a,
	group_id: impl Into<GroupId> + 'a,
	children: Vec<String>,
) -> impl Future<Output = AppRes<()>> + 'a
{
	file_model::delete_files_for_group(app_id, group_id, children)
}

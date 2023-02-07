mod local_storage;

use std::env;

use async_trait::async_trait;
use rustgram::{Request, Response};
use tokio::sync::OnceCell;

use crate::error::SentcCoreError;

static FILE_HANDLER: OnceCell<Box<dyn FileHandler>> = OnceCell::const_new();

#[async_trait]
pub trait FileHandler: Send + Sync
{
	async fn get_part(&self, part_id: &str, content_type: Option<&str>) -> Result<Response, SentcCoreError>;

	async fn upload_part(&self, req: Request, part_id: &str, max_chunk_size: usize) -> Result<usize, SentcCoreError>;

	async fn delete_part(&self, part_id: &str) -> Result<(), SentcCoreError>;

	async fn delete_parts(&self, parts: &[String]) -> Result<(), SentcCoreError>;
}

pub async fn init_storage()
{
	let storage = env::var("BACKEND_STORAGE").unwrap();

	if storage.as_str() == "0" {
		FILE_HANDLER.get_or_init(local_storage::init_storage).await;
	}
}

pub fn get_local_storage(path: String) -> Box<dyn FileHandler>
{
	Box::new(local_storage::LocalStorage::new(path))
}

pub async fn get_part(part_id: &str) -> Result<Response, SentcCoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.get_part(part_id, None).await
}

pub async fn upload_part(req: Request, part_id: &str, max_chunk_size: usize) -> Result<usize, SentcCoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.upload_part(req, part_id, max_chunk_size).await
}

pub async fn delete_part(part_id: &str) -> Result<(), SentcCoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.delete_part(part_id).await
}

pub async fn delete_parts(parts: &[String]) -> Result<(), SentcCoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.delete_parts(parts).await
}

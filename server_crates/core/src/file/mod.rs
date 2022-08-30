mod local_storage;

use std::env;

use async_trait::async_trait;
use rustgram::{Request, Response};
use tokio::sync::OnceCell;

use crate::error::CoreError;

static FILE_HANDLER: OnceCell<Box<dyn FileHandler>> = OnceCell::const_new();

#[async_trait]
pub trait FileHandler: Send + Sync
{
	async fn get_part(&self, part_id: &str) -> Result<Response, CoreError>;

	async fn upload_part(&self, req: Request, part_id: &str, max_chunk_size: usize) -> Result<usize, CoreError>;

	async fn delete_part(&self, part_id: &str) -> Result<(), CoreError>;

	async fn delete_parts(&self, parts: &Vec<String>) -> Result<(), CoreError>;
}

pub async fn init_storage()
{
	let storage = env::var("BACKEND_STORAGE").unwrap();

	if storage.as_str() == "0" {
		FILE_HANDLER.get_or_init(local_storage::init_storage).await;
	}
}

pub async fn get_part(part_id: &str) -> Result<Response, CoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.get_part(part_id).await
}

pub async fn upload_part(req: Request, part_id: &str, max_chunk_size: usize) -> Result<usize, CoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.upload_part(req, part_id, max_chunk_size).await
}

pub async fn delete_part(part_id: &str) -> Result<(), CoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.delete_part(part_id).await
}

pub async fn delete_parts(parts: &Vec<String>) -> Result<(), CoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.delete_parts(parts).await
}

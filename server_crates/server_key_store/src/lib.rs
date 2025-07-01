mod local_key_store;
#[cfg(feature = "s3_key_storage")]
mod s3_storage;

use std::collections::HashMap;
use std::env;

use async_trait::async_trait;
use rustgram_server_util::res::AppRes;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::local_key_store::LocalKeyStore;

#[derive(Serialize, Deserialize)]
pub struct KeyStorage
{
	pub id: String,
	pub key: String,
}

#[async_trait]
pub trait KeyStore: Send + Sync
{
	async fn get(&self, keys: &[String]) -> AppRes<HashMap<String, String>>;

	async fn upload_key(&self, keys: Vec<KeyStorage>) -> AppRes<()>;

	async fn delete_key(&self, keys: &[String]) -> AppRes<()>;
}

static FILE_HANDLER: OnceCell<Box<dyn KeyStore>> = OnceCell::const_new();

pub async fn init_key_store()
{
	let storage = env::var("BACKEND_KEY_STORAGE").unwrap_or_else(|_| "0".to_string());

	if storage.as_str() == "0" {
		FILE_HANDLER.get_or_init(init_local_key_store).await;
	}

	#[cfg(feature = "s3_key_storage")]
	if storage.as_str() == "1" {
		//aws storage
		FILE_HANDLER.get_or_init(s3_storage::init_s3_storage).await;
	}
}

async fn init_local_key_store() -> Box<dyn KeyStore>
{
	let path = env::var("LOCAL_KEY_STORAGE").unwrap();

	Box::new(LocalKeyStore::new(path))
}

pub async fn get_keys(keys: &[String]) -> AppRes<HashMap<String, String>>
{
	let handler = FILE_HANDLER.get().unwrap();
	handler.get(keys).await
}

pub async fn upload_key(keys: Vec<KeyStorage>) -> AppRes<()>
{
	let handler = FILE_HANDLER.get().unwrap();
	handler.upload_key(keys).await
}

pub async fn delete_key(keys: &[String]) -> AppRes<()>
{
	let handler = FILE_HANDLER.get().unwrap();
	handler.delete_key(keys).await
}

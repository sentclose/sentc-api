use std::env;

use async_trait::async_trait;
use tokio::sync::OnceCell;

mod array_cache;

static CACHE: OnceCell<Box<dyn Cache<String>>> = OnceCell::const_new();

#[async_trait]
pub trait Cache<T: 'static + Clone>: Send + Sync
{
	async fn get(&self, key: &str) -> Option<T>;

	async fn add(&self, key: String, value: T, ttl: usize);
}

pub async fn init_cache()
{
	let cache = env::var("CACHE").unwrap();

	if cache.as_str() == "1" {
		CACHE.get_or_init(array_cache::init_cache::<String>).await;
	}
}

pub async fn get(key: &str) -> Option<String>
{
	let cache = CACHE.get().unwrap();

	cache.get(key).await
}

pub async fn add(key: String, value: String, ttl: usize)
{
	let cache = CACHE.get().unwrap();

	cache.add(key, value, ttl).await
}

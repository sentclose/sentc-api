use std::env;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

mod array_cache;

static CACHE: OnceCell<Box<dyn Cache<String>>> = OnceCell::const_new();

#[async_trait]
pub trait Cache<T: 'static + Clone>: Send + Sync
{
	async fn get(&self, key: &str) -> Option<T>;

	async fn add(&self, key: String, value: T, ttl: usize);

	async fn delete(&self, key: &str);
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

pub async fn delete(key: &str)
{
	let cache = CACHE.get().unwrap();

	cache.delete(key).await;
}

#[derive(Serialize, Deserialize)]
pub enum CacheVariant<T>
{
	Some(T),
	None,
}

pub static DEFAULT_TTL: usize = 60 * 60; //1h (60 sec * 60 min)
pub static LONG_TTL: usize = 60 * 60 * 24; //24 h

pub static JWT_CACHE: &'static str = "jwtcache";
pub static APP_TOKEN_CACHE: &'static str = "apptokencache";

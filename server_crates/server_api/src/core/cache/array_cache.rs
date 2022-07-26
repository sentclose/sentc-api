use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::core::cache::Cache;
use crate::core::get_time_in_sec;

pub struct CacheData<T: 'static + Clone>
{
	value: T,
	ttl: usize,
}

/**
# Simple Array Cache with Multithreaded support

https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
with RwLock instead of Mutex
 */
pub struct ArrayCache<T: 'static + Clone>
{
	//https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html
	cache: RwLock<HashMap<String, CacheData<T>>>,
}

impl<T: 'static + Clone> ArrayCache<T>
{
	pub fn new() -> Self
	{
		Self {
			cache: RwLock::new(HashMap::<String, CacheData<T>>::new()),
		}
	}
}

#[async_trait]
impl<T: 'static + Clone + Send + Sync> Cache<T> for ArrayCache<T>
{
	async fn get(&self, key: &str) -> Option<T>
	{
		let cache = self.cache.read().await;

		match cache.get(key) {
			Some(v) => {
				if v.ttl < get_time_in_sec().unwrap() as usize {
					return None;
				}

				Some(v.value.clone())
			},
			None => None,
		}
	}

	async fn add(&self, key: String, value: T, ttl: usize)
	{
		self.cache.write().await.insert(
			key,
			CacheData {
				ttl,
				value,
			},
		);
	}

	async fn delete(&self, key: &str)
	{
		self.cache.write().await.remove(key);
	}
}

/**
Init th cache as async

Must be async for RwLock from tokio.
*/
pub async fn init_cache<T: 'static + Clone + Send + Sync>() -> Box<dyn Cache<T>>
{
	Box::new(ArrayCache::new())
}

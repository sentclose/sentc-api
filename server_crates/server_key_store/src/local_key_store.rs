use std::collections::HashMap;

use async_trait::async_trait;
use rustgram_server_util::error::{server_err, server_err_owned, CoreErrorCodes};
use rustgram_server_util::res::AppRes;
use tokio::fs::{remove_file, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{KeyStorage, KeyStore};

pub struct LocalKeyStore
{
	path: String,
}

impl LocalKeyStore
{
	pub fn new(path: String) -> Self
	{
		LocalKeyStore {
			path,
		}
	}
}

#[async_trait]
impl KeyStore for LocalKeyStore
{
	async fn get(&self, keys: &[String]) -> AppRes<HashMap<String, String>>
	{
		//iterate over all keys and fetch each key
		let mut output = HashMap::with_capacity(keys.len());

		for key in keys {
			let path = format!("{}/{}", self.path, key);

			let mut file = File::open(path).await.map_err(|e| {
				server_err_owned(
					400,
					CoreErrorCodes::FileLocalOpen,
					format!("error in open file: {}", e),
					None,
				)
			})?;

			let mut buffer = String::new();

			file.read_to_string(&mut buffer).await.map_err(|e| {
				server_err_owned(
					400,
					CoreErrorCodes::FileLocalOpen,
					format!("error in open file: {}", e),
					None,
				)
			})?;

			//store it with the key id
			output.insert(key.to_string(), buffer);
		}

		Ok(output)
	}

	async fn upload_key(&self, keys: Vec<KeyStorage>) -> AppRes<()>
	{
		for KeyStorage {
			key,
			id,
		} in keys
		{
			let path = format!("{}/{}", self.path, id);

			let mut file = File::create(path)
				.await
				.map_err(|_e| server_err(400, CoreErrorCodes::FileLocalOpen, "error in creating file"))?;

			file.write_all(key.as_bytes()).await.map_err(|e| {
				server_err_owned(
					400,
					CoreErrorCodes::DbBulkInsert,
					"Can't save the file".to_string(),
					Some(format!("error in saving a file: {}, error: {}", id, e)),
				)
			})?;
		}

		Ok(())
	}

	async fn delete_key(&self, keys: &[String]) -> AppRes<()>
	{
		for key in keys {
			let path = format!("{}/{}", self.path, key);

			remove_file(path).await.map_err(|e| {
				server_err_owned(
					400,
					CoreErrorCodes::FileRemove,
					"Can't save the file".to_string(),
					Some(format!("error in removing file: {}, error: {}", key, e)),
				)
			})?;
		}

		Ok(())
	}
}

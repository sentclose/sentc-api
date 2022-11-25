use std::env;

use async_trait::async_trait;
use futures::StreamExt;
use hyper::Body;
use rustgram::{Request, Response};
use tokio::fs::{remove_file, File};
use tokio::io::AsyncWriteExt;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::error::{CoreError, CoreErrorCodes};
use crate::file::FileHandler;

pub struct LocalStorage
{
	path: String,
}

impl LocalStorage
{
	pub fn new(path: String) -> Self
	{
		Self {
			path,
		}
	}

	async fn remove_file(&self, path: &str) -> Result<(), CoreError>
	{
		remove_file(path).await.map_err(|e| {
			CoreError::new(
				400,
				CoreErrorCodes::FileRemove,
				"Can't save the file".to_string(),
				Some(format!("error in removing file: {}, error: {}", path, e)),
			)
		})
	}
}

#[async_trait]
impl FileHandler for LocalStorage
{
	async fn get_part(&self, part_id: &str, content_type: Option<&str>) -> Result<Response, CoreError>
	{
		let path = self.path.to_string() + "/" + part_id;

		let file = File::open(path.as_str()).await.map_err(|e| {
			CoreError::new(
				400,
				CoreErrorCodes::FileLocalOpen,
				format!("error in open file: {}", e),
				None,
			)
		})?;

		let stream = FramedRead::new(file, BytesCodec::new());
		let body = Body::wrap_stream(stream);

		let content_type = content_type.unwrap_or("application/octet-stream");

		hyper::Response::builder()
			.header("Content-Type", content_type)
			.header("Access-Control-Allow-Origin", "*")
			.body(body)
			.map_err(|_e| {
				CoreError::new(
					400,
					CoreErrorCodes::DbBulkInsert,
					"Can't download the file".to_string(),
					None,
				)
			})
	}

	async fn upload_part(&self, req: Request, part_id: &str, max_chunk_size: usize) -> Result<usize, CoreError>
	{
		let path = self.path.to_string() + "/" + part_id;

		let mut file = File::create(path.as_str()).await.map_err(|_e| {
			CoreError::new(
				400,
				CoreErrorCodes::FileLocalOpen,
				"error in creating file".to_string(),
				None,
			)
		})?;

		let mut body = req.into_body();
		let mut size: usize = 0;

		while let Some(bytes) = body.next().await {
			let bytes = match bytes {
				Ok(b) => b,
				Err(_e) => {
					continue;
				},
			};

			let b_len = bytes.len();

			if b_len + size > max_chunk_size {
				self.remove_file(path.as_str()).await?;

				return Err(CoreError::new(
					400,
					CoreErrorCodes::FileSave,
					format!(
						"File chunk is too large to upload. The max chunk size is: {}",
						max_chunk_size
					),
					None,
				));
			}

			file.write_all(&bytes).await.map_err(|e| {
				CoreError::new(
					400,
					CoreErrorCodes::DbBulkInsert,
					"Can't save the file".to_string(),
					Some(format!("error in saving a file: {}, error: {}", part_id, e)),
				)
			})?;

			size += b_len;
		}

		Ok(size)
	}

	async fn delete_part(&self, part_id: &str) -> Result<(), CoreError>
	{
		let path = self.path.to_string() + "/" + part_id;

		self.remove_file(path.as_str()).await
	}

	#[allow(clippy::single_match)]
	async fn delete_parts(&self, parts: &[String]) -> Result<(), CoreError>
	{
		//delete every part
		for part in parts {
			let path = self.path.to_string() + "/" + part.as_str();

			//ignore the error here, maybe later just print out the error to std
			match self.remove_file(path.as_str()).await {
				Ok(_) => {},
				Err(_) => {},
			}
		}

		Ok(())
	}
}

pub async fn init_storage() -> Box<dyn FileHandler>
{
	let path = env::var("LOCAL_STORAGE_PATH").unwrap();

	Box::new(LocalStorage::new(path))
}

use std::collections::HashMap;
use std::env;

use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::Client;
use rustgram_server_util::error::{server_err_owned, CoreErrorCodes};
use rustgram_server_util::input_helper::bytes_to_json;
use rustgram_server_util::res::AppRes;

use crate::{KeyStorage, KeyStore};

pub(super) async fn init_s3_storage() -> Box<dyn KeyStore>
{
	let bucket_name = env::var("AWS_S3_BUCKET").unwrap();

	let local = env::var("AWS_S3_LOCAL").unwrap_or("0".to_string());
	if local == "1" {
		let store = S3KeyStore::new_localstack(bucket_name, "http://localhost:4566")
			.await
			.unwrap();

		return Box::new(store);
	}

	let region = env::var("AWS_DEFAULT_REGION");

	let store = if let Ok(region) = region {
		S3KeyStore::new_with_region(bucket_name, &region)
			.await
			.unwrap()
	} else {
		S3KeyStore::new(bucket_name).await.unwrap()
	};

	Box::new(store)
}

// S3 implementation
pub struct S3KeyStore
{
	client: Client,
	bucket_name: String,
}

impl S3KeyStore
{
	pub async fn new(bucket_name: String) -> AppRes<Self>
	{
		let config = aws_config::defaults(BehaviorVersion::latest()).load().await;

		let client = Client::new(&config);

		Ok(Self {
			client,
			bucket_name,
		})
	}

	pub async fn new_with_region(bucket_name: String, region: &str) -> AppRes<Self>
	{
		let config = aws_config::defaults(BehaviorVersion::latest())
			.region(Region::new(region.to_string()))
			.load()
			.await;

		let client = Client::new(&config);

		Ok(Self {
			client,
			bucket_name,
		})
	}

	// Helper method to generate S3 object key from id
	fn get_object_key(&self, id: &str) -> String
	{
		format!("keys/{}.json", id)
	}

	// Constructor for LocalStack
	pub async fn new_localstack(bucket_name: String, endpoint_url: &str) -> AppRes<Self>
	{
		let creds = Credentials::new(
			"test",       // access_key
			"test",       // secret_key
			None,         // session_token
			None,         // expiration
			"localstack", // provider_name
		);

		let config = aws_config::defaults(BehaviorVersion::latest())
			.region(Region::new("us-east-1"))
			.credentials_provider(creds)
			.endpoint_url(endpoint_url)
			.load()
			.await;

		let s3_config = aws_sdk_s3::config::Builder::from(&config)
			.force_path_style(true)
			.build();

		let client = Client::from_conf(s3_config);

		// Create a bucket if it doesn't exist (LocalStack)
		let _ = client.create_bucket().bucket(&bucket_name).send().await; // Ignore errors if bucket already exists

		Ok(Self {
			client,
			bucket_name,
		})
	}
}

#[async_trait]
impl KeyStore for S3KeyStore
{
	async fn get(&self, keys: &[String]) -> AppRes<HashMap<String, String>>
	{
		let mut result = HashMap::new();

		// Fetch keys concurrently
		let mut tasks = Vec::new();

		for key_id in keys {
			let client = self.client.clone();
			let bucket = self.bucket_name.clone();
			let object_key = self.get_object_key(key_id);
			let key_id = key_id.clone();

			let task = tokio::spawn(async move {
				let response = client
					.get_object()
					.bucket(bucket)
					.key(object_key)
					.send()
					.await;

				match response {
					Ok(output) => {
						let body = output
							.body
							.collect()
							.await
							.map_err(|e| server_err_owned(400, CoreErrorCodes::FileLocalOpen, e.to_string(), None))?;
						let key_data: KeyStorage = bytes_to_json(&body.into_bytes())?;
						Ok((key_id, key_data.key))
					},
					Err(e) => {
						// Check if it's a "not found" error
						if let aws_sdk_s3::error::SdkError::ServiceError(service_err) = &e {
							if service_err.err().is_no_such_key() {
								return Ok((key_id, String::new())); // Return empty string for missing keys
							}
						}
						Err(server_err_owned(
							400,
							CoreErrorCodes::FileLocalOpen,
							e.to_string(),
							None,
						))
					},
				}
			});

			tasks.push(task);
		}

		// Wait for all tasks to complete
		for task in tasks {
			let (key_id, key_value) = task
				.await
				.map_err(|e| server_err_owned(400, CoreErrorCodes::FileLocalOpen, e.to_string(), None))??;
			if !key_value.is_empty() {
				result.insert(key_id, key_value);
			}
		}

		Ok(result)
	}

	async fn upload_key(&self, keys: Vec<KeyStorage>) -> AppRes<()>
	{
		// Upload keys concurrently
		let mut tasks = Vec::new();

		for key_storage in keys {
			let client = self.client.clone();
			let bucket = self.bucket_name.clone();
			let object_key = self.get_object_key(&key_storage.id);

			let task = tokio::spawn(async move {
				let json_data =
					serde_json::to_vec(&key_storage).map_err(|e| server_err_owned(400, CoreErrorCodes::DbBulkInsert, e.to_string(), None))?;

				client
					.put_object()
					.bucket(bucket)
					.key(object_key)
					.body(json_data.into())
					.content_type("application/json")
					.send()
					.await
					.map_err(|e| server_err_owned(400, CoreErrorCodes::DbBulkInsert, e.to_string(), None))?;

				Ok(())
			});

			tasks.push(task);
		}

		// Wait for all uploads to complete
		for task in tasks {
			task.await
				.map_err(|e| server_err_owned(400, CoreErrorCodes::DbBulkInsert, e.to_string(), None))??;
		}

		Ok(())
	}

	async fn delete_key(&self, keys: &[String]) -> AppRes<()>
	{
		if keys.is_empty() {
			return Ok(());
		}

		// AWS S3 batch delete supports up to 1000 objects per request
		const BATCH_SIZE: usize = 1000;

		for chunk in keys.chunks(BATCH_SIZE) {
			if chunk.len() == 1 {
				// Single delete for small batches (sometimes faster)
				let object_key = self.get_object_key(&chunk[0]);
				self.client
					.delete_object()
					.bucket(&self.bucket_name)
					.key(object_key)
					.send()
					.await
					.map_err(|e| server_err_owned(400, CoreErrorCodes::FileRemove, e.to_string(), None))?;
			} else {
				// Use batch delete for multiple objects
				let mut delete_objects = Vec::new();

				for key_id in chunk {
					let object_key = self.get_object_key(key_id);
					let obj_identifier = aws_sdk_s3::types::ObjectIdentifier::builder()
						.key(object_key)
						.build()
						.map_err(|e| server_err_owned(400, CoreErrorCodes::FileRemove, e.to_string(), None))?;
					delete_objects.push(obj_identifier);
				}

				let delete_request = aws_sdk_s3::types::Delete::builder()
					.set_objects(Some(delete_objects))
					.quiet(true) // Don't return info about successful deletions
					.build()
					.map_err(|e| server_err_owned(400, CoreErrorCodes::FileRemove, e.to_string(), None))?;

				let response = self
					.client
					.delete_objects()
					.bucket(&self.bucket_name)
					.delete(delete_request)
					.send()
					.await
					.map_err(|e| server_err_owned(400, CoreErrorCodes::FileRemove, e.to_string(), None))?;

				// Check for any errors in the batch delete
				let errors = response.errors();
				if !errors.is_empty() {
					let error_messages: Vec<String> = errors
						.iter()
						.map(|e| {
							format!(
								"Key: {}, Code: {}, Message: {}",
								e.key().unwrap_or("unknown"),
								e.code().unwrap_or("unknown"),
								e.message().unwrap_or("unknown")
							)
						})
						.collect();

					return Err(server_err_owned(
						400,
						CoreErrorCodes::FileRemove,
						format!("Batch delete errors: {}", error_messages.join("; ")),
						None,
					));
				}
			}
		}

		Ok(())
	}
}

/*
// Example usage
#[tokio::main]
async fn main() -> AppRes<()>
{
	// Initialize the global store
	initialize_s3_store("my-key-storage-bucket".to_string()).await?;

	let store = get_store().unwrap();

	// Upload some keys
	let keys_to_upload = vec![
		KeyStorage {
			id: "user1".to_string(),
			key: "secret_key_1".to_string(),
		},
		KeyStorage {
			id: "user2".to_string(),
			key: "secret_key_2".to_string(),
		},
	];

	store.upload_key(keys_to_upload).await?;

	// Retrieve keys
	let key_ids = vec!["user1".to_string(), "user2".to_string()];
	let retrieved_keys = store.get(&key_ids).await?;

	for (id, key) in retrieved_keys {
		println!("ID: {}, Key: {}", id, key);
	}

	// Delete keys
	let keys_to_delete = vec!["user1", "user2"];
	store.delete_key(&keys_to_delete).await?;

	Ok(())
}


 */

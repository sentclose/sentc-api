use rand::RngCore;
use reqwest::header::AUTHORIZATION;
use sentc_crypto::group::GroupKeyData;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::UserData;
use sentc_crypto_common::file::FileData;
use sentc_crypto_common::GroupId;
use server_api::sentc_file_worker;
use server_api_common::app::AppRegisterOutput;
use server_api_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{auth_header, create_app, create_group, create_test_customer, create_test_user, customer_delete, get_group, get_url};

mod test_fn;

pub struct GroupData
{
	keys: Vec<GroupKeyData>,
	id: GroupId,
}

pub struct TestData
{
	pub user_data: UserData,
	pub username: String,
	pub user_pw: String,
	pub app_data: AppRegisterOutput,
	pub customer_data: CustomerDoneLoginOutput,
	pub group_data: GroupData,
	pub files: Vec<FileData>,
	pub file_ids: Vec<String>,
	pub test_file_large: Vec<u8>,
	pub test_file_big: Vec<u8>,
	pub test_file_mid: Vec<u8>,
	pub test_file_small: Vec<u8>,
}

static TEST_SMALL_FILE_SIZE: usize = 1024 * 1024; //1mb (to test files without chunking)
static TEST_MID_FILE_SIZE: usize = 1024 * 1024 * 5; //5 mb exact chunking
static TEST_BIG_FILE_SIZE: usize = 1024 * 1024 * 15; //15 mb
static TEST_LARGE_FILE_SIZE: usize = 1024 * 1024 * 100; //100 mb

static TEST_STATE: OnceCell<RwLock<TestData>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_state()
{
	dotenv::dotenv().ok();

	let (_, customer_data) = create_test_customer("hello@test5.com", "12345").await;

	let customer_jwt = &customer_data.user_keys.jwt;

	//create here an app
	let app_data = create_app(customer_jwt).await;

	let secret_token = app_data.secret_token.to_string();
	let public_token = app_data.public_token.to_string();

	let user_pw = "12345";
	let username = "hello5";

	let (_user_id, key_data) = create_test_user(secret_token.as_str(), public_token.as_str(), username, user_pw).await;

	let group_id = create_group(
		secret_token.as_str(),
		&key_data.keys.public_key,
		None,
		key_data.jwt.as_str(),
	)
	.await;

	let group_keys = get_group(
		secret_token.as_str(),
		key_data.jwt.as_str(),
		group_id.as_str(),
		&key_data.keys.private_key,
		false,
	)
	.await
	.1;

	let group_data = GroupData {
		keys: group_keys,
		id: group_id,
	};

	let mut rng = rand::thread_rng();

	//create "files"
	let mut test_file_large = vec![0u8; TEST_SMALL_FILE_SIZE];
	rng.try_fill_bytes(&mut test_file_large).unwrap();

	let mut test_file_big = vec![0u8; TEST_SMALL_FILE_SIZE];
	rng.try_fill_bytes(&mut test_file_big).unwrap();

	let mut test_file_mid = vec![0u8; TEST_SMALL_FILE_SIZE];
	rng.try_fill_bytes(&mut test_file_mid).unwrap();

	let mut test_file_small = vec![0u8; TEST_SMALL_FILE_SIZE];
	rng.try_fill_bytes(&mut test_file_small).unwrap();

	TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(TestData {
					user_data: key_data,
					username: username.to_string(),
					user_pw: user_pw.to_string(),
					app_data,
					customer_data,
					group_data,
					files: vec![],
					file_ids: vec![],
					test_file_large,
					test_file_big,
					test_file_mid,
					test_file_small,
				})
			}
		})
		.await;
}

#[tokio::test]
async fn test_10_upload_small_file_for_non_target()
{
	let mut state = TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/file".to_string());

	let file_key = &state.group_data.keys[0].group_key;

	//normally create a new sym key for a file but here it is ok
	let input = sentc_crypto::file::prepare_register_file(file_key, None, sentc_crypto::sdk_common::file::BelongsToType::None).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//session id
	let (file_id, session_id) = sentc_crypto::file::done_register_file(body.as_str()).unwrap();

	let file = &state.test_file_small;

	//no chunk needed
	let encrypted_small_file = sentc_crypto::crypto::encrypt_symmetric(file_key, file, None).unwrap();

	//upload the file
	let url = get_url("api/v1/file/part/".to_string() + session_id.as_str() + "/1/true");

	//buffered req
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.body(encrypted_small_file)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();

	state.file_ids = vec![file_id];
}

#[tokio::test]
async fn test_11_download_small_file_non_target()
{
	let mut state = TEST_STATE.get().unwrap().write().await;
	let file_id = &state.file_ids[0];
	let file_key = &state.group_data.keys[0].group_key;

	//download the file info
	let url = get_url("api/v1/file/".to_string() + file_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let file_data: FileData = handle_server_response(body.as_str()).unwrap();

	assert_eq!(file_data.key_id, file_key.key_id);

	//download the part
	let url = get_url("api/v1/file/part/".to_string() + &file_data.part_list[0].part_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.send()
		.await
		.unwrap();

	let buffer = res.bytes().await.unwrap().to_vec();

	let decrypted_file = sentc_crypto::crypto::decrypt_symmetric(file_key, &buffer, None).unwrap();

	let file = &state.test_file_small;

	assert_eq!(file.len(), decrypted_file.len());

	for i in 0..decrypted_file.len() {
		let org = file[i];
		let decrypted = decrypted_file[i];

		assert_eq!(org, decrypted);
	}

	state.files = vec![file_data];
}

#[tokio::test]
async fn test_12_upload_small_file_for_group()
{
	let mut state = TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/group/".to_string() + &state.group_data.id + "/file");

	let file_key = &state.group_data.keys[0].group_key;

	//normally create a new sym key for a file but here it is ok
	let input = sentc_crypto::file::prepare_register_file(
		file_key,
		Some(state.group_data.id.to_string()),
		sentc_crypto::sdk_common::file::BelongsToType::Group,
	)
	.unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//session id
	let (file_id, session_id) = sentc_crypto::file::done_register_file(body.as_str()).unwrap();

	let file = &state.test_file_small;

	//no chunk needed
	let encrypted_small_file = sentc_crypto::crypto::encrypt_symmetric(file_key, file, None).unwrap();

	//upload the file
	let url = get_url("api/v1/file/part/".to_string() + session_id.as_str() + "/1/true");

	//buffered req
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.body(encrypted_small_file)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();

	state.file_ids.push(file_id);
}

#[tokio::test]
async fn test_13_get_file_in_group()
{
	let mut state = TEST_STATE.get().unwrap().write().await;
	let file_id = &state.file_ids[1];
	let file_key = &state.group_data.keys[0].group_key;

	//download the file info
	let url = get_url("api/v1/group/".to_string() + &state.group_data.id + "/file/" + file_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let file_data: FileData = handle_server_response(body.as_str()).unwrap();

	assert_eq!(file_data.key_id, file_key.key_id);

	//download the part
	let url = get_url("api/v1/file/part/".to_string() + &file_data.part_list[0].part_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.send()
		.await
		.unwrap();

	let buffer = res.bytes().await.unwrap().to_vec();

	let decrypted_file = sentc_crypto::crypto::decrypt_symmetric(file_key, &buffer, None).unwrap();

	let file = &state.test_file_small;

	assert_eq!(file.len(), decrypted_file.len());

	for i in 0..decrypted_file.len() {
		let org = file[i];
		let decrypted = decrypted_file[i];

		assert_eq!(org, decrypted);
	}

	state.files.push(file_data);
}

/**
TODO
	- handle files with chunks
	 - when uploading file in group, check if non group member can access the file (create a 2nd user)
	 - check if non group member can upload files in a group
	 - when deleting a group, check if the file is marked as deleted
	 - when uploading the file in a child group check if member from parent group got access
	 - when deleting the parent group check if file from the child group is marked as deleted
*/

#[tokio::test]
async fn zz_clean_up()
{
	let state = TEST_STATE.get().unwrap().read().await;
	customer_delete(state.customer_data.user_keys.jwt.as_str()).await;
}

#[tokio::test]
async fn zzz_test_worker_delete()
{
	dotenv::dotenv().ok();

	//override the path for this process because test files are exec in sub dir
	std::env::set_var("LOCAL_STORAGE_PATH", "../../storage");
	std::env::set_var("DB_PATH", std::env::var("DB_PATH_TEST").unwrap());

	server_api::start().await;

	sentc_file_worker::start().await.unwrap();
}

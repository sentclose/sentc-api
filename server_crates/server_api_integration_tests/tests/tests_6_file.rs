use rand::RngCore;
use reqwest::header::AUTHORIZATION;
#[cfg(feature = "mysql")]
use rustgram_server_util::db::mysql_async_export::prelude::Queryable;
use sentc_crypto::sdk_common::file::FileData;
use sentc_crypto::sdk_utils::cryptomat::SymKeyCrypto;
use sentc_crypto::sdk_utils::error::SdkUtilError;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::SdkError;
use sentc_crypto_common::file::FileRegisterOutput;
use sentc_crypto_common::{GroupId, ServerOutput};
use server_dashboard_common::app::AppRegisterOutput;
use server_dashboard_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
	add_user_by_invite,
	auth_header,
	create_app,
	create_group,
	create_test_customer,
	create_test_user,
	customer_delete,
	get_and_decrypt_file_part,
	get_and_decrypt_file_part_start,
	get_file,
	get_group,
	get_server_error_from_normal_res,
	get_url,
	TestFileEncryptor,
	TestGroupKeyData,
	TestKeyGenerator,
	TestUserDataInt,
};

mod test_fn;

pub struct GroupData
{
	keys: Vec<TestGroupKeyData>,
	keys_2: Vec<TestGroupKeyData>, //for the 2nd user
	id: GroupId,
}

pub struct TestData
{
	//user 1
	pub user_data: TestUserDataInt,
	pub username: String,
	pub user_pw: String,

	//user 2
	pub user_data_1: TestUserDataInt,
	pub username_1: String,
	pub user_pw_1: String,

	pub app_data: AppRegisterOutput,
	pub customer_data: CustomerDoneLoginOutput,

	//parent group
	pub group_data: GroupData,

	//child group
	pub child_group_data: GroupData,

	pub files: Vec<FileData>,
	pub file_ids: Vec<String>,
	pub test_file_big: Vec<u8>,
	pub test_file_mid: Vec<u8>,
	pub test_file_small: Vec<u8>,
}

const TEST_SMALL_FILE_SIZE: usize = 1024 * 1024; //1mb (to test files without chunking)
const TEST_MID_FILE_SIZE: usize = 1024 * 1024 * 5; //5 mb exact chunking
const TEST_BIG_FILE_SIZE: usize = 1024 * 1024 * 14; //14 mb

static TEST_STATE: OnceCell<RwLock<TestData>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_state()
{
	dotenv::from_filename("sentc.env").ok();

	let (_, customer_data) = create_test_customer("hello@test6.com", "12345").await;

	let customer_jwt = &customer_data.verify.jwt;

	//create here an app
	let app_data = create_app(customer_jwt).await;

	let secret_token = app_data.secret_token.to_string();
	let public_token = app_data.public_token.to_string();

	let user_pw = "12345";
	let username = "hello6";
	let user_pw_1 = "12345";
	let username_1 = "hello61";

	let (_user_id, key_data) = create_test_user(secret_token.as_str(), public_token.as_str(), username, user_pw).await;
	let (_user_id, key_data_1) = create_test_user(secret_token.as_str(), public_token.as_str(), username_1, user_pw_1).await;

	//create parent group

	let group_id = create_group(
		secret_token.as_str(),
		&key_data.user_keys[0].public_key,
		None,
		key_data.jwt.as_str(),
	)
	.await;

	let group_keys = get_group(
		secret_token.as_str(),
		key_data.jwt.as_str(),
		group_id.as_str(),
		&key_data.user_keys[0].private_key,
		false,
	)
	.await
	.1;

	let group_data = GroupData {
		keys: group_keys,
		keys_2: vec![],
		id: group_id.to_string(),
	};

	//create child group

	let group_id = create_group(
		secret_token.as_str(),
		&group_data.keys[0].public_group_key,
		Some(group_id),
		key_data.jwt.as_str(),
	)
	.await;

	let group_keys = get_group(
		secret_token.as_str(),
		key_data.jwt.as_str(),
		group_id.as_str(),
		&group_data.keys[0].private_group_key,
		false,
	)
	.await
	.1;

	let child_group_data = GroupData {
		keys: group_keys,
		keys_2: vec![],
		id: group_id,
	};

	let mut rng = rand::thread_rng();

	//create "files"
	let mut test_file_big = vec![0u8; TEST_BIG_FILE_SIZE];
	rng.try_fill_bytes(&mut test_file_big).unwrap();

	let mut test_file_mid = vec![0u8; TEST_MID_FILE_SIZE];
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
					user_data_1: key_data_1,
					username_1: username_1.to_string(),
					user_pw_1: user_pw_1.to_string(),
					app_data,
					customer_data,
					group_data,
					child_group_data,
					files: vec![],
					file_ids: vec![],
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

	let (file_key, encrypted_key) = TestKeyGenerator::generate_non_register_sym_key(&state.group_data.keys[0].group_key).unwrap();
	let encrypted_key_str = encrypted_key.to_string().unwrap();

	//normally create a new sym key for a file but here it is ok
	let (input, _) = sentc_crypto::file::prepare_register_file(
		encrypted_key.master_key_id.clone(),
		&file_key,
		encrypted_key_str,
		None,
		sentc_crypto::sdk_common::file::BelongsToType::None,
		Some("Hello".to_string()),
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
	let (encrypted_small_file, next_key) = TestFileEncryptor::encrypt_file_part_start(&file_key, file, None).unwrap();

	//should not upload the file from a different user
	let (encrypted_small_file_1, _) = TestFileEncryptor::encrypt_file_part_start(&file_key, file, None).unwrap();

	let url = get_url("api/v1/file/part/".to_string() + session_id.as_str() + "/1/true");

	//buffered req
	let client = reqwest::Client::new();
	match client
		.post(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data_1.jwt.as_str()))
		.body(encrypted_small_file_1)
		.send()
		.await
	{
		Ok(res) => {
			let body = res.text().await.unwrap();

			let server_err = get_server_error_from_normal_res(&body);

			assert_eq!(server_err, 510);
		},
		Err(_e) => {
			//err here is ok because the server can just close the connection for wrong parts
		},
	}

	//finally upload the file_______________________________________________________________________
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

	//should not upload a part with finished session
	let (encrypted_small_file_2, _) = TestFileEncryptor::encrypt_file_part(&next_key, file, None).unwrap();

	let url = get_url("api/v1/file/part/".to_string() + session_id.as_str() + "/1/true");

	//buffered req
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.body(encrypted_small_file_2)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, 510);

	//save the data
	state.file_ids = vec![file_id];
}

#[tokio::test]
async fn test_11_download_small_file_non_target()
{
	let mut state = TEST_STATE.get().unwrap().write().await;
	let file_id = &state.file_ids[0];
	let group_key = &state.group_data.keys[0].group_key;

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

	let file_key = TestKeyGenerator::done_fetch_sym_key(group_key, &file_data.encrypted_key, true).unwrap();

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

	let (decrypted_file, _) = TestFileEncryptor::decrypt_file_part_start(&file_key, &buffer, None).unwrap();

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
async fn test_12_update_file_name()
{
	let state = TEST_STATE.get().unwrap().read().await;
	let file_id = &state.file_ids[0];
	let file_key = &state.group_data.keys[0].group_key;

	let input = sentc_crypto::file::prepare_file_name_update(file_key, Some("Hello 123".to_string())).unwrap();

	let url = get_url("api/v1/file/".to_string() + file_id);
	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();

	//should get the file info with the new name
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

	let decrypted_name = file_key
		.decrypt_string(file_data.encrypted_file_name.unwrap().as_str(), None)
		.unwrap();

	assert_eq!(decrypted_name, "Hello 123");
}

#[tokio::test]
async fn test_12_upload_small_file_for_group()
{
	let mut state = TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/group/".to_string() + &state.group_data.id + "/file");

	let (file_key, encrypted_key) = TestKeyGenerator::generate_non_register_sym_key(&state.group_data.keys[0].group_key).unwrap();
	let encrypted_key_str = encrypted_key.to_string().unwrap();

	//normally create a new sym key for a file but here it is ok
	let (input, _) = sentc_crypto::file::prepare_register_file(
		encrypted_key.master_key_id,
		&file_key,
		encrypted_key_str,
		Some(state.group_data.id.to_string()),
		sentc_crypto::sdk_common::file::BelongsToType::Group,
		None,
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
	let (encrypted_small_file, _) = TestFileEncryptor::encrypt_file_part_start(&file_key, file, None).unwrap();

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
	let group_key = &state.group_data.keys[0].group_key;

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

	let file_key = TestKeyGenerator::done_fetch_sym_key(group_key, &file_data.encrypted_key, true).unwrap();

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

	let (decrypted_file, _) = TestFileEncryptor::decrypt_file_part_start(&file_key, &buffer, None).unwrap();

	let file = &state.test_file_small;

	assert_eq!(file.len(), decrypted_file.len());

	for i in 0..decrypted_file.len() {
		let org = file[i];
		let decrypted = decrypted_file[i];

		assert_eq!(org, decrypted);
	}

	state.files.push(file_data);
}

#[tokio::test]
async fn test_14_get_not_files_in_group_without_group_access()
{
	let state = TEST_STATE.get().unwrap().read().await;
	let file_id = &state.file_ids[1];

	//user 1 is not in this group
	let url = get_url("api/v1/group/".to_string() + &state.group_data.id + "/file/" + file_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data_1.jwt.as_str()))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<FileData>::from_string(body.as_str()).unwrap();

	assert!(!out.status);
}

#[tokio::test]
async fn test_15_not_upload_file_in_a_group_without_access()
{
	let state = TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/group/".to_string() + &state.group_data.id + "/file");

	let (file_key, encrypted_key) = TestKeyGenerator::generate_non_register_sym_key(&state.group_data.keys[0].group_key).unwrap();
	let encrypted_key_str = encrypted_key.to_string().unwrap();

	//normally create a new sym key for a file but here it is ok
	let (input, _) = sentc_crypto::file::prepare_register_file(
		encrypted_key.master_key_id,
		&file_key,
		encrypted_key_str,
		Some(state.group_data.id.to_string()),
		sentc_crypto::sdk_common::file::BelongsToType::Group,
		None,
	)
	.unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(state.user_data_1.jwt.as_str()))
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<FileRegisterOutput>::from_string(body.as_str()).unwrap();

	assert!(!out.status);

	match sentc_crypto::file::done_register_file(body.as_str()) {
		Ok(_) => {
			panic!("should be an error")
		},
		Err(e) => {
			match e {
				SdkError::Util(SdkUtilError::ServerErr(s, _)) => {
					assert_eq!(s, 310); //no group access error
				},
				_ => {
					panic!("should be server error")
				},
			}
		},
	}
}

#[tokio::test]
async fn test_16_no_file_access_when_for_child_group_user()
{
	//push 2nd user to the child group
	let mut state = TEST_STATE.get().unwrap().write().await;

	let child_group = &state.child_group_data;

	let (_user_group_data_2, group_keys) = add_user_by_invite(
		state.app_data.secret_token.as_str(),
		state.user_data.jwt.as_str(),
		child_group.id.as_str(),
		&child_group.keys,
		state.user_data_1.user_id.as_str(),
		state.user_data_1.jwt.as_str(),
		&state.user_data_1.user_keys[0].exported_public_key,
		&state.user_data_1.user_keys[0].private_key,
	)
	.await;

	state.child_group_data.keys_2 = group_keys;

	//should not access the file of the parent group
	let file_id = &state.file_ids[1];

	//user 1 is not in this group
	let url = get_url("api/v1/group/".to_string() + &state.group_data.id + "/file/" + file_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data_1.jwt.as_str()))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<FileData>::from_string(body.as_str()).unwrap();

	assert!(!out.status);
}

#[tokio::test]
async fn test_17_file_access_from_parent_to_child_group()
{
	//create file in the child group by the direct member
	let mut state = TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/group/".to_string() + &state.child_group_data.id + "/file");

	let (file_key, encrypted_key) = TestKeyGenerator::generate_non_register_sym_key(&state.child_group_data.keys[0].group_key).unwrap();
	let encrypted_key_str = encrypted_key.to_string().unwrap();

	//normally create a new sym key for a file but here it is ok
	let (input, _) = sentc_crypto::file::prepare_register_file(
		encrypted_key.master_key_id,
		&file_key,
		encrypted_key_str,
		Some(state.child_group_data.id.to_string()),
		sentc_crypto::sdk_common::file::BelongsToType::Group,
		None,
	)
	.unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(state.user_data_1.jwt.as_str()))
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
	let (encrypted_small_file, _) = TestFileEncryptor::encrypt_file_part_start(&file_key, file, None).unwrap();

	//upload the file
	let url = get_url("api/v1/file/part/".to_string() + session_id.as_str() + "/1/true");

	//buffered req
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data_1.jwt.as_str()))
		.body(encrypted_small_file)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();

	state.file_ids.push(file_id);
}

#[tokio::test]
async fn test_18_access_the_child_group_file()
{
	//access from the direct member
	let mut state = TEST_STATE.get().unwrap().write().await;
	let file_id = &state.file_ids[2];
	let group_key = &state.child_group_data.keys[0].group_key;

	//download the file info
	let file_data = get_file(
		file_id,
		state.user_data_1.jwt.as_str(),
		state.app_data.public_token.as_str(),
		Some(&state.child_group_data.id),
	)
	.await;

	let file_key = TestKeyGenerator::done_fetch_sym_key(group_key, &file_data.encrypted_key, true).unwrap();

	//download the part
	let (decrypted_file, _) = get_and_decrypt_file_part_start(
		&file_data.part_list[0].part_id,
		state.user_data_1.jwt.as_str(),
		state.app_data.public_token.as_str(),
		&file_key,
	)
	.await;

	let file = &state.test_file_small;

	assert_eq!(file.len(), decrypted_file.len());

	for i in 0..decrypted_file.len() {
		let org = file[i];
		let decrypted = decrypted_file[i];

		assert_eq!(org, decrypted);
	}

	//now get the file as a parent group member_______________________
	let file_data = get_file(
		file_id,
		state.user_data.jwt.as_str(),
		state.app_data.public_token.as_str(),
		Some(&state.child_group_data.id),
	)
	.await;

	//download the part
	let (decrypted_file, _) = get_and_decrypt_file_part_start(
		&file_data.part_list[0].part_id,
		state.user_data.jwt.as_str(),
		state.app_data.public_token.as_str(),
		&file_key,
	)
	.await;

	let file = &state.test_file_small;

	assert_eq!(file.len(), decrypted_file.len());

	for i in 0..decrypted_file.len() {
		let org = file[i];
		let decrypted = decrypted_file[i];

		assert_eq!(org, decrypted);
	}

	state.files.push(file_data);
}

//__________________________________________________________________________________________________
//handle file delete

struct FileDeleteCheck
{
	pub status: i32,
	pub deleted_at: u128,
}

#[cfg(feature = "mysql")]
impl rustgram_server_util::db::mysql_async_export::prelude::FromRow for FileDeleteCheck
{
	fn from_row_opt(
		mut row: rustgram_server_util::db::mysql_async_export::Row,
	) -> Result<Self, rustgram_server_util::db::mysql_async_export::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			status: rustgram_server_util::take_or_err!(row, 0, i32),
			deleted_at: rustgram_server_util::take_or_err!(row, 1, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl rustgram_server_util::db::FromSqliteRow for FileDeleteCheck
{
	fn from_row_opt(row: &rustgram_server_util::db::rusqlite_export::Row) -> Result<Self, rustgram_server_util::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			status: rustgram_server_util::take_or_err!(row, 0),
			deleted_at: rustgram_server_util::take_or_err_u128!(row, 1),
		})
	}
}

#[tokio::test]
async fn test_19_not_delete_file_without_access()
{
	let state = TEST_STATE.get().unwrap().read().await;

	let file_id_to_delete = &state.file_ids[0];

	let url = get_url("api/v1/file/".to_string() + file_id_to_delete);

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(state.user_data_1.jwt.as_str()))
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, 521);
}

#[tokio::test]
async fn test_20_delete_a_file()
{
	//index 0 the non target file
	let state = TEST_STATE.get().unwrap().read().await;

	let file_id_to_delete = &state.file_ids[0];

	let url = get_url("api/v1/file/".to_string() + file_id_to_delete);

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();

	//check if the deleted file is marked as deleted
	dotenv::from_filename("sentc.env").ok();
	std::env::set_var("DB_PATH", std::env::var("DB_PATH_TEST").unwrap());

	/*
		Use single conn here because of the pool conn limit from mysql
	*/
	#[cfg(feature = "mysql")]
	{
		let user = std::env::var("DB_USER").unwrap();
		let pw = std::env::var("DB_PASS").unwrap();
		let mysql_host = std::env::var("DB_HOST").unwrap();
		let db = std::env::var("DB_NAME").unwrap();

		let mut conn = rustgram_server_util::db::mysql_async_export::Conn::new(
			rustgram_server_util::db::mysql_async_export::Opts::try_from(format!("mysql://{}:{}@{}/{}", user, pw, mysql_host, db).as_str()).unwrap(),
		)
		.await
		.unwrap();

		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id = ?";

		let deleted_file = conn
			.exec_first::<FileDeleteCheck, _, _>(sql, rustgram_server_util::set_params!(file_id_to_delete.to_string()))
			.await
			.unwrap();

		let deleted_file = deleted_file.unwrap();

		let time = rustgram_server_util::get_time().unwrap();

		//the deleted time should be in the past
		assert!(deleted_file.deleted_at <= time);

		//from file model: FILE_STATUS_TO_DELETE
		assert_eq!(deleted_file.status, 0);

		//check the other files (from the group) this files should not be deleted
		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id NOT LIKE ?";

		let deleted_file = conn
			.exec::<FileDeleteCheck, _, _>(sql, rustgram_server_util::set_params!(file_id_to_delete.to_string()))
			.await
			.unwrap();

		//parent group got a file and child group
		assert_eq!(deleted_file.len(), 2);

		for file in deleted_file {
			assert_eq!(file.status, 1);
			assert_eq!(file.deleted_at, 0);
		}

		drop(conn);
	}

	#[cfg(feature = "sqlite")]
	{
		rustgram_server_util::db::init_db().await;

		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id = ?";

		let deleted_file: Option<FileDeleteCheck> =
			rustgram_server_util::db::query_first(sql, rustgram_server_util::set_params!(file_id_to_delete.to_string()))
				.await
				.unwrap();
		let deleted_file = deleted_file.unwrap();

		let time = rustgram_server_util::get_time().unwrap();

		//the deleted time should be in the past
		assert!(!(deleted_file.deleted_at > time));

		//from file model: FILE_STATUS_TO_DELETE
		assert_eq!(deleted_file.status, 0);

		//check the other files (from the group) this files should not be deleted
		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id NOT LIKE ?";

		let deleted_file: Vec<FileDeleteCheck> =
			rustgram_server_util::db::query(sql, rustgram_server_util::set_params!(file_id_to_delete.to_string()))
				.await
				.unwrap();

		//parent group got a file and child group
		assert_eq!(deleted_file.len(), 2);

		for file in deleted_file {
			assert_eq!(file.status, 1);
			assert_eq!(file.deleted_at, 0);
		}
	}
}

#[tokio::test]
async fn test_21_delete_file_via_group_delete()
{
	//delete the parent group.
	let state = TEST_STATE.get().unwrap().read().await;
	let group_id = &state.group_data.id;

	let creator = &state.user_data;
	let secret_token = &state.app_data.secret_token;

	let url = get_url("api/v1/group/".to_owned() + group_id);
	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(creator.jwt.as_str()))
		.header("x-sentc-app-token", secret_token)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	handle_general_server_response(body.as_str()).unwrap();

	//check if the group file was deleted (files[1])
	let file_id = &state.file_ids[1];
	let child_group_file = &state.file_ids[2];

	dotenv::from_filename("sentc.env").ok();
	std::env::set_var("DB_PATH", std::env::var("DB_PATH_TEST").unwrap());

	#[cfg(feature = "mysql")]
	{
		let user = std::env::var("DB_USER").unwrap();
		let pw = std::env::var("DB_PASS").unwrap();
		let mysql_host = std::env::var("DB_HOST").unwrap();
		let db = std::env::var("DB_NAME").unwrap();

		let mut conn = rustgram_server_util::db::mysql_async_export::Conn::new(
			rustgram_server_util::db::mysql_async_export::Opts::try_from(format!("mysql://{}:{}@{}/{}", user, pw, mysql_host, db).as_str()).unwrap(),
		)
		.await
		.unwrap();

		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id = ?";

		let deleted_file = conn
			.exec_first::<FileDeleteCheck, _, _>(sql, rustgram_server_util::set_params!(file_id.to_string()))
			.await
			.unwrap();

		let deleted_file = deleted_file.unwrap();

		let time = rustgram_server_util::get_time().unwrap();

		//the deleted time should be in the past
		assert!(deleted_file.deleted_at <= time);

		//from file model: FILE_STATUS_TO_DELETE
		assert_eq!(deleted_file.status, 0);

		//check if the child group file was deleted (files[2])

		//check the other files (from the group) this files should not be deleted
		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id = ?";

		let deleted_file = conn
			.exec_first::<FileDeleteCheck, _, _>(sql, rustgram_server_util::set_params!(child_group_file.to_string()))
			.await
			.unwrap();

		let deleted_file = deleted_file.unwrap();

		//the deleted time should be in the past
		assert!(deleted_file.deleted_at <= time);

		//from file model: FILE_STATUS_TO_DELETE
		assert_eq!(deleted_file.status, 0);

		drop(conn);
	}

	#[cfg(feature = "sqlite")]
	{
		rustgram_server_util::db::init_db().await;

		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id = ?";

		let deleted_file: Option<FileDeleteCheck> =
			rustgram_server_util::db::query_first(sql, rustgram_server_util::set_params!(file_id.to_string()))
				.await
				.unwrap();
		let deleted_file = deleted_file.unwrap();

		let time = rustgram_server_util::get_time().unwrap();

		//the deleted time should be in the past
		assert!(!(deleted_file.deleted_at > time));

		//from file model: FILE_STATUS_TO_DELETE
		assert_eq!(deleted_file.status, 0);

		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id = ?";

		let deleted_file: Option<FileDeleteCheck> =
			rustgram_server_util::db::query_first(sql, rustgram_server_util::set_params!(child_group_file.to_string()))
				.await
				.unwrap();

		let deleted_file = deleted_file.unwrap();

		//the deleted time should be in the past
		assert!(!(deleted_file.deleted_at > time));

		//from file model: FILE_STATUS_TO_DELETE
		assert_eq!(deleted_file.status, 0);
	}
}

//__________________________________________________________________________________________________
//chunk file

#[tokio::test]
async fn test_30_chunked_filed()
{
	//create and upload a bigger file
	let mut state = TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/file".to_string());

	let (file_key, encrypted_key) = TestKeyGenerator::generate_non_register_sym_key(&state.group_data.keys[0].group_key).unwrap();
	let encrypted_key_str = encrypted_key.to_string().unwrap();

	//normally create a new sym key for a file but here it is ok
	let (input, _) = sentc_crypto::file::prepare_register_file(
		encrypted_key.master_key_id,
		&file_key,
		encrypted_key_str,
		None,
		sentc_crypto::sdk_common::file::BelongsToType::None,
		None,
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

	let file = &state.test_file_big;

	//should not upload a too large part
	let too_large_part = &file[0..1024 * 1024 * 6];
	let (encrypted_part, _next_key) = TestFileEncryptor::encrypt_file_part_start(&file_key, too_large_part, None).unwrap();
	let url = get_url("api/v1/file/part/".to_string() + session_id.as_str() + "/0/false");

	let client = reqwest::Client::new();
	match client
		.post(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.body(encrypted_part)
		.send()
		.await
	{
		Ok(res) => {
			let body = res.text().await.unwrap();

			let server_err = get_server_error_from_normal_res(&body);

			assert_eq!(server_err, 502);
		},
		Err(_e) => {
			//err here is ok because the server can just close the connection for wrong parts
		},
	}

	//upload the real parts
	//chunk the file, use 4 instead of 5 mb because of the little overhead of the encryption
	let chunk_size = 1024 * 1024 * 4;

	let mut start = 0;
	let mut end = chunk_size;
	let mut current_chunk = 0;

	let mut next_key = None;

	while start < file.len() {
		current_chunk += 1;

		//vec slice fill panic if the end is bigger than the length
		let used_end = if end > file.len() { file.len() } else { end };

		let part = &file[start..used_end];

		start = end;
		end = start + chunk_size;
		let is_end = start >= file.len();

		let encrypted_res = if current_chunk == 1 {
			TestFileEncryptor::encrypt_file_part_start(&file_key, part, None).unwrap()
		} else {
			TestFileEncryptor::encrypt_file_part(&next_key.unwrap(), part, None).unwrap()
		};

		let encrypted_part = encrypted_res.0;
		next_key = Some(encrypted_res.1);

		let url = get_url(
			"api/v1/file/part/".to_string() + session_id.as_str() + "/" + current_chunk.to_string().as_str() + "/" + is_end.to_string().as_str(),
		);

		let client = reqwest::Client::new();
		let res = client
			.post(url)
			.header("x-sentc-app-token", state.app_data.public_token.as_str())
			.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
			.body(encrypted_part)
			.send()
			.await
			.unwrap();

		let body = res.text().await.unwrap();

		handle_general_server_response(body.as_str()).unwrap();
	}

	state.file_ids.push(file_id);
}

#[tokio::test]
async fn test_31_download_chunked_file()
{
	let mut state = TEST_STATE.get().unwrap().write().await;
	let file_id = &state.file_ids[3];
	let group_key = &state.group_data.keys[0].group_key;

	let file_data = get_file(
		file_id,
		state.user_data.jwt.as_str(),
		state.app_data.public_token.as_str(),
		None,
	)
	.await;

	let file_key = TestKeyGenerator::done_fetch_sym_key(group_key, &file_data.encrypted_key, true).unwrap();

	let parts = &file_data.part_list;

	let mut file: Vec<u8> = Vec::new();

	let mut next_key = None;

	for (i, part) in parts.iter().enumerate() {
		let part_id = &part.part_id;

		//should all internal storage
		let decrypted_part = if i == 0 {
			get_and_decrypt_file_part_start(
				part_id,
				state.user_data.jwt.as_str(),
				state.app_data.public_token.as_str(),
				&file_key,
			)
			.await
		} else {
			get_and_decrypt_file_part(
				part_id,
				state.user_data.jwt.as_str(),
				state.app_data.public_token.as_str(),
				&next_key.unwrap(),
			)
			.await
		};

		next_key = Some(decrypted_part.1);
		let mut decrypted_part = decrypted_part.0;

		file.append(&mut decrypted_part);
	}

	let org_file = &state.test_file_big;

	assert_eq!(file.len(), org_file.len());

	for i in 0..file.len() {
		let org = org_file[i];
		let decrypted = file[i];

		assert_eq!(org, decrypted);
	}

	state.files.push(file_data);
}

#[tokio::test]
async fn zz_clean_up()
{
	let state = TEST_STATE.get().unwrap().read().await;
	customer_delete(state.customer_data.verify.jwt.as_str()).await;
}

#[tokio::test]
async fn zzz_test_worker_delete()
{
	dotenv::from_filename("sentc.env").ok();

	//override the path for this process because test files are exec in sub dir
	std::env::set_var("LOCAL_STORAGE_PATH", "../../storage");
	std::env::set_var("DB_PATH", std::env::var("DB_PATH_TEST").unwrap());

	server_api_common::start().await;

	server_api_file::file_worker::start().await.unwrap();
}

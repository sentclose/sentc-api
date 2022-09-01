#[cfg(feature = "mysql")]
use mysql_async::prelude::Queryable;
use rand::RngCore;
use reqwest::header::AUTHORIZATION;
use sentc_crypto::group::GroupKeyData;
use sentc_crypto::sdk_common::file::FileData;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::{SdkError, UserData};
use sentc_crypto_common::file::FileRegisterOutput;
use sentc_crypto_common::{GroupId, ServerOutput};
use server_api::sentc_file_worker;
use server_api_common::app::AppRegisterOutput;
use server_api_common::customer::CustomerDoneLoginOutput;
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
	get_file,
	get_group,
	get_url,
};

mod test_fn;

pub struct GroupData
{
	keys: Vec<GroupKeyData>,
	keys_2: Vec<GroupKeyData>, //for the 2nd user
	id: GroupId,
}

pub struct TestData
{
	//user 1
	pub user_data: UserData,
	pub username: String,
	pub user_pw: String,

	//user 2
	pub user_data_1: UserData,
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

static TEST_SMALL_FILE_SIZE: usize = 1024 * 1024; //1mb (to test files without chunking)
static TEST_MID_FILE_SIZE: usize = 1024 * 1024 * 5; //5 mb exact chunking
static TEST_BIG_FILE_SIZE: usize = 1024 * 1024 * 14; //14 mb

static TEST_STATE: OnceCell<RwLock<TestData>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_state()
{
	dotenv::dotenv().ok();

	let (_, customer_data) = create_test_customer("hello@test6.com", "12345").await;

	let customer_jwt = &customer_data.user_keys.jwt;

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

	//should not upload the file from a different user
	let encrypted_small_file_1 = sentc_crypto::crypto::encrypt_symmetric(file_key, file, None).unwrap();

	let url = get_url("api/v1/file/part/".to_string() + session_id.as_str() + "/1/true");

	//buffered req
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data_1.jwt.as_str()))
		.body(encrypted_small_file_1)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	match handle_general_server_response(body.as_str()) {
		Ok(_) => {
			panic!("Should be an error");
		},
		Err(e) => {
			match e {
				SdkError::ServerErr(s, _msg) => {
					assert_eq!(s, 510); //session not found error for the
				},
				_ => {
					panic!("Should be server error")
				},
			}
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
	let encrypted_small_file_2 = sentc_crypto::crypto::encrypt_symmetric(file_key, file, None).unwrap();

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

	match handle_general_server_response(body.as_str()) {
		Ok(_) => {
			panic!("Should be an error");
		},
		Err(e) => {
			match e {
				SdkError::ServerErr(s, _msg) => {
					assert_eq!(s, 510); //session not found error
				},
				_ => {
					panic!("Should be server error")
				},
			}
		},
	}

	//save the data
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

	assert_eq!(out.status, false);
}

#[tokio::test]
async fn test_15_not_upload_file_in_a_group_without_access()
{
	let state = TEST_STATE.get().unwrap().read().await;

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
		.header(AUTHORIZATION, auth_header(state.user_data_1.jwt.as_str()))
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out = ServerOutput::<FileRegisterOutput>::from_string(body.as_str()).unwrap();

	assert_eq!(out.status, false);

	match sentc_crypto::file::done_register_file(body.as_str()) {
		Ok(_) => {
			panic!("should be an error")
		},
		Err(e) => {
			match e {
				SdkError::ServerErr(s, _msg) => {
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
		&state.user_data_1.keys.exported_public_key,
		&state.user_data_1.keys.private_key,
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

	assert_eq!(out.status, false);
}

#[tokio::test]
async fn test_17_file_access_from_parent_to_child_group()
{
	//create file in the child group by the direct member
	let mut state = TEST_STATE.get().unwrap().write().await;

	let url = get_url("api/v1/group/".to_string() + &state.child_group_data.id + "/file");

	let file_key = &state.child_group_data.keys[0].group_key;

	//normally create a new sym key for a file but here it is ok
	let input = sentc_crypto::file::prepare_register_file(
		file_key,
		Some(state.child_group_data.id.to_string()),
		sentc_crypto::sdk_common::file::BelongsToType::Group,
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
	let encrypted_small_file = sentc_crypto::crypto::encrypt_symmetric(file_key, file, None).unwrap();

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
	let file_key = &state.child_group_data.keys[0].group_key;

	//download the file info
	let file_data = get_file(
		file_id,
		state.user_data_1.jwt.as_str(),
		state.app_data.public_token.as_str(),
		Some(&state.child_group_data.id),
	)
	.await;

	//download the part
	let decrypted_file = get_and_decrypt_file_part(
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
	let decrypted_file = get_and_decrypt_file_part(
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
impl mysql_async::prelude::FromRow for FileDeleteCheck
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self {
			status: server_core::take_or_err!(row, 0, i32),
			deleted_at: server_core::take_or_err!(row, 1, u128),
		})
	}
}

#[cfg(feature = "sqlite")]
impl server_core::db::FromSqliteRow for FileDeleteCheck
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, server_core::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		let time: String = server_core::take_or_err!(row, 1);
		let time: u128 = time.parse().map_err(|e| {
			server_core::db::FormSqliteRowError {
				msg: format!("err in db fetch: {:?}", e),
			}
		})?;

		Ok(Self {
			status: server_core::take_or_err!(row, 0),
			deleted_at: time,
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

	match handle_general_server_response(body.as_str()) {
		Ok(_) => {
			panic!("should be error");
		},
		Err(e) => {
			match e {
				SdkError::ServerErr(c, _) => {
					assert_eq!(c, 521);
				},
				_ => {
					panic!("should be server error");
				},
			}
		},
	}
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
	dotenv::dotenv().ok();
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

		let mut conn =
			mysql_async::Conn::new(mysql_async::Opts::try_from(format!("mysql://{}:{}@{}/{}", user, pw, mysql_host, db).as_str()).unwrap())
				.await
				.unwrap();

		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id = ?";

		let deleted_file = conn
			.exec_first::<FileDeleteCheck, _, _>(sql, server_core::set_params!(file_id_to_delete.to_string()))
			.await
			.unwrap();

		let deleted_file = deleted_file.unwrap();

		let time = server_core::get_time().unwrap();

		//the deleted time should be in the past
		assert!(!(deleted_file.deleted_at > time));

		//from file model: FILE_STATUS_TO_DELETE
		assert_eq!(deleted_file.status, 0);

		//check the other files (from the group) this files should not be deleted
		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id NOT LIKE ?";

		let deleted_file = conn
			.exec::<FileDeleteCheck, _, _>(sql, server_core::set_params!(file_id_to_delete.to_string()))
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
		server_core::db::init_db().await;

		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id = ?";

		let deleted_file: Option<FileDeleteCheck> = server_core::db::query_first(sql, server_core::set_params!(file_id_to_delete.to_string()))
			.await
			.unwrap();
		let deleted_file = deleted_file.unwrap();

		let time = server_core::get_time().unwrap();

		//the deleted time should be in the past
		assert!(!(deleted_file.deleted_at > time));

		//from file model: FILE_STATUS_TO_DELETE
		assert_eq!(deleted_file.status, 0);

		//check the other files (from the group) this files should not be deleted
		//language=SQL
		let sql = "SELECT status,delete_at FROM sentc_file WHERE id NOT LIKE ?";

		let deleted_file: Vec<FileDeleteCheck> = server_core::db::query(sql, server_core::set_params!(file_id_to_delete.to_string()))
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

//__________________________________________________________________________________________________
//chunk file

#[tokio::test]
async fn test_30_chunked_filed()
{
	//create and upload a bigger file
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

	let file = &state.test_file_big;

	//should not upload a too large part
	let too_large_part = &file[0..1024 * 1024 * 6];
	let encrypted_part = sentc_crypto::crypto::encrypt_symmetric(file_key, too_large_part, None).unwrap();
	let url = get_url("api/v1/file/part/".to_string() + session_id.as_str() + "/0/false");

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

	match handle_general_server_response(body.as_str()) {
		Ok(_) => {
			panic!("Should be an error")
		},
		Err(e) => {
			match e {
				SdkError::ServerErr(s, _) => {
					assert_eq!(s, 502);
				},
				_ => panic!("must be server error"),
			}
		},
	}

	//upload the real parts
	//chunk the file, use 4 instead of 5 mb because of the little overhead of the encryption
	let chunk_size = 1024 * 1024 * 4;

	let mut start = 0;
	let mut end = chunk_size;
	let mut current_chunk = 0;

	while start < file.len() {
		current_chunk += 1;

		//vec slice fill panic if the end is bigger than the length
		let used_end = if end > file.len() { file.len() } else { end };

		let part = &file[start..used_end];

		start = end;
		end = start + chunk_size;
		let is_end = start >= file.len();

		let encrypted_part = sentc_crypto::crypto::encrypt_symmetric(file_key, part, None).unwrap();

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
	let file_key = &state.group_data.keys[0].group_key;

	let file_data = get_file(
		file_id,
		state.user_data.jwt.as_str(),
		state.app_data.public_token.as_str(),
		None,
	)
	.await;

	let parts = &file_data.part_list;

	let mut file: Vec<u8> = Vec::new();

	for part in parts {
		let part_id = &part.part_id;

		//should all internal storage
		let mut decrypted_part = get_and_decrypt_file_part(
			part_id,
			state.user_data.jwt.as_str(),
			state.app_data.public_token.as_str(),
			&file_key,
		)
		.await;

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

/**
TODO
	 - when deleting a group, check if the file is marked as deleted
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

#[ignore]
#[tokio::test]
async fn test_0_large_file()
{
	dotenv::dotenv().ok();

	//prepare
	let file_size = 1024 * 1024 * 104; //104 mb

	let (_, customer_data) = create_test_customer("hello@test61.com", "12345").await;
	let customer_jwt = &customer_data.user_keys.jwt;
	let app_data = create_app(customer_jwt).await;
	let secret_token = app_data.secret_token.to_string();
	let public_token = app_data.public_token.to_string();
	let user_pw = "12345";
	let username = "hello6";
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

	let mut rng = rand::thread_rng();

	//______________________________________________________________________________________________
	//create "files"
	let mut file = vec![0u8; file_size];
	rng.try_fill_bytes(&mut file).unwrap();

	//______________________________________________________________________________________________
	//upload the file
	let url = get_url("api/v1/file".to_string());

	let file_key = &group_keys[0].group_key;
	let input = sentc_crypto::file::prepare_register_file(file_key, None, sentc_crypto::sdk_common::file::BelongsToType::None).unwrap();

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(key_data.jwt.as_str()))
		.header("x-sentc-app-token", public_token.as_str())
		.body(input)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	//session id
	let (file_id, session_id) = sentc_crypto::file::done_register_file(body.as_str()).unwrap();

	//______________________________________________________________________________________________
	//chunk the file
	let chunk_size = 1024 * 1024 * 4;

	let mut start = 0;
	let mut end = chunk_size;
	let mut current_chunk = 0;

	while start < file.len() {
		current_chunk += 1;

		//vec slice fill panic if the end is bigger than the length
		let used_end = if end > file.len() { file.len() } else { end };

		let part = &file[start..used_end];

		start = end;
		end = start + chunk_size;
		let is_end = start >= file.len();

		let encrypted_part = sentc_crypto::crypto::encrypt_symmetric(file_key, part, None).unwrap();

		let url = get_url(
			"api/v1/file/part/".to_string() + session_id.as_str() + "/" + current_chunk.to_string().as_str() + "/" + is_end.to_string().as_str(),
		);

		let client = reqwest::Client::new();
		let res = client
			.post(url)
			.header("x-sentc-app-token", public_token.as_str())
			.header(AUTHORIZATION, auth_header(key_data.jwt.as_str()))
			.body(encrypted_part)
			.send()
			.await
			.unwrap();

		let body = res.text().await.unwrap();

		handle_general_server_response(body.as_str()).unwrap();
	}

	//______________________________________________________________________________________________
	//test download the large file
	let file_data = get_file(&file_id, key_data.jwt.as_str(), public_token.as_str(), None).await;

	let parts = &file_data.part_list;

	let mut downloaded_file: Vec<u8> = Vec::new();

	for part in parts {
		let part_id = &part.part_id;

		//should all internal storage
		let mut decrypted_part = get_and_decrypt_file_part(part_id, key_data.jwt.as_str(), public_token.as_str(), &file_key).await;

		downloaded_file.append(&mut decrypted_part);
	}

	assert_eq!(file.len(), downloaded_file.len());

	for i in 0..downloaded_file.len() {
		let org = file[i];
		let decrypted = file[i];

		assert_eq!(org, decrypted);
	}

	//______________________________________________________________________________________________
	//mark it as delete
	customer_delete(customer_data.user_keys.jwt.as_str()).await;

	//delete the file

	//override the path for this process because test files are exec in sub dir
	std::env::set_var("LOCAL_STORAGE_PATH", "../../storage");
	std::env::set_var("DB_PATH", std::env::var("DB_PATH_TEST").unwrap());

	server_api::start().await;

	sentc_file_worker::start().await.unwrap();
}

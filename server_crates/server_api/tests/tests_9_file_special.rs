use rand::RngCore;
use reqwest::header::AUTHORIZATION;
use sentc_crypto::entities::group::GroupKeyData;
use sentc_crypto::entities::user::UserDataInt;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto_common::file::FilePartRegisterOutput;
use sentc_crypto_common::{FileId, PartId};
use server_api::sentc_file_worker;
use server_api_common::app::{AppFileOptionsInput, AppOptions, AppRegisterInput, AppRegisterOutput, FILE_STORAGE_OWN};
use server_api_common::customer::CustomerDoneLoginOutput;
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{
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
};

mod test_fn;

static TEST_STATE: OnceCell<RwLock<TestData>> = OnceCell::const_new();

pub struct TestData
{
	//user 1
	pub user_data: UserDataInt,
	pub username: String,
	pub user_pw: String,

	pub app_data: AppRegisterOutput,
	pub customer_data: CustomerDoneLoginOutput,
	pub file_part_ids: Vec<PartId>,
	pub file_id: FileId,

	pub keys: Vec<GroupKeyData>,
}

async fn init_app()
{
	let (_, customer_data) = create_test_customer("hello@test7.com", "12345").await;

	let url = get_url("api/v1/customer/app".to_owned());

	let input = AppRegisterInput {
		identifier: None,
		options: AppOptions::default(),
		file_options: AppFileOptionsInput {
			file_storage: FILE_STORAGE_OWN,
			storage_url: Some(format!("http://127.0.0.1:{}/file_part/delete", 3003)),
			auth_token: Some("abc".to_string()),
		},
		group_options: Default::default(),
	};

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(&customer_data.user_keys.jwt))
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: AppRegisterOutput = handle_server_response(body.as_str()).unwrap();

	let secret_token = out.secret_token.to_string();
	let public_token = out.public_token.to_string();

	let user_pw = "12345";
	let username = "hello6";

	let (_user_id, key_data) = create_test_user(secret_token.as_str(), public_token.as_str(), username, user_pw).await;

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

	TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(TestData {
					user_data: key_data,
					username: username.to_string(),
					user_pw: user_pw.to_string(),
					app_data: out,
					customer_data,
					file_part_ids: vec![],
					file_id: "".to_string(),
					keys: group_keys,
				})
			}
		})
		.await;
}

async fn create_file()
{
	let mut state = TEST_STATE.get().unwrap().write().await;
	let group_key = &state.keys[0].group_key;

	let (file_key, encrypted_key) = sentc_crypto::crypto::generate_non_register_sym_key(group_key).unwrap();
	let encrypted_key_str = encrypted_key.to_string().unwrap();

	let (input, _) = sentc_crypto::file::prepare_register_file(
		encrypted_key.master_key_id,
		&file_key,
		encrypted_key_str,
		None,
		sentc_crypto::sdk_common::file::BelongsToType::None,
		Some("Hello".to_string()),
	)
	.unwrap();

	let url = get_url("api/v1/file".to_string());

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

	//not use the upload fn because the storage options must block it
	let url = get_url("api/v1/file/part/".to_string() + session_id.as_str() + "/1/true");

	let file_dummy: Vec<u8> = vec![1, 2, 3, 4];

	//buffered req
	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header("x-sentc-app-token", state.app_data.public_token.as_str())
		.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
		.body(file_dummy)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let server_err = get_server_error_from_normal_res(&body);

	assert_eq!(server_err, 520);

	//register 502 parts for pagination
	for i in 0..502 {
		let url = get_url("api/v1/file/part/".to_string() + session_id.as_str() + "/" + i.to_string().as_str() + "/false");

		let client = reqwest::Client::new();
		let res = client
			.patch(url)
			.header("x-sentc-app-token", state.app_data.secret_token.as_str())
			.header(AUTHORIZATION, auth_header(state.user_data.jwt.as_str()))
			.send()
			.await
			.unwrap();

		let body = res.text().await.unwrap();

		let out: FilePartRegisterOutput = handle_server_response(body.as_str()).unwrap();

		//make res to the dummy server to save the part ids
		let client = reqwest::Client::new();
		let res = client
			.get(format!("http://127.0.0.1:{}/file_part/upload/{}", 3003, out.part_id))
			.send()
			.await
			.unwrap();

		let _body = res.text().await.unwrap();
	}

	state.file_id = file_id;
}

async fn delete_file()
{
	//don't delete file via customer delete because the customer must do it self
	let state = TEST_STATE.get().unwrap().read().await;
	let file_id = &state.file_id;

	let url = get_url("api/v1/file/".to_string() + file_id);

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
}

async fn delete_file_worker()
{
	//override the path for this process because test files are exec in sub dir
	std::env::set_var("LOCAL_STORAGE_PATH", "../../storage");
	std::env::set_var("DB_PATH", std::env::var("DB_PATH_TEST").unwrap());

	server_api::start().await;

	sentc_file_worker::start().await.unwrap();
}

async fn clean_up()
{
	let state = TEST_STATE.get().unwrap().read().await;
	customer_delete(state.customer_data.user_keys.jwt.as_str()).await;
}

#[ignore]
#[tokio::test]
async fn file_external()
{
	dotenv::from_filename("sentc.env").ok();

	init_app().await;

	create_file().await;

	delete_file().await;

	delete_file_worker().await;

	clean_up().await;
}

#[ignore]
#[tokio::test]
async fn test_0_large_file()
{
	dotenv::from_filename("sentc.env").ok();

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

	let mut rng = rand::thread_rng();

	//______________________________________________________________________________________________
	//create "files"
	let mut file = vec![0u8; file_size];
	rng.try_fill_bytes(&mut file).unwrap();

	//______________________________________________________________________________________________
	//upload the file
	let url = get_url("api/v1/file".to_string());

	let (file_key, encrypted_key) = sentc_crypto::crypto::generate_non_register_sym_key(&group_keys[0].group_key).unwrap();
	let encrypted_key_str = encrypted_key.to_string().unwrap();

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

	let mut next_key = sentc_crypto::sdk_core::SymKey::Aes(Default::default());

	while start < file.len() {
		current_chunk += 1;

		//vec slice fill panic if the end is bigger than the length
		let used_end = if end > file.len() { file.len() } else { end };

		let part = &file[start..used_end];

		start = end;
		end = start + chunk_size;
		let is_end = start >= file.len();

		let encrypted_res = if current_chunk == 1 {
			sentc_crypto::file::encrypt_file_part_start(&file_key, part, None).unwrap()
		} else {
			sentc_crypto::file::encrypt_file_part(&next_key, part, None).unwrap()
		};

		let encrypted_part = encrypted_res.0;
		next_key = encrypted_res.1;

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

	let mut next_key = sentc_crypto::sdk_core::SymKey::Aes(Default::default());

	for (i, part) in parts.iter().enumerate() {
		let part_id = &part.part_id;

		//should all internal storage
		let decrypted_part = if i == 0 {
			get_and_decrypt_file_part_start(part_id, &key_data.jwt, &public_token, &file_key).await
		} else {
			get_and_decrypt_file_part(part_id, &key_data.jwt, &public_token, &next_key).await
		};

		next_key = decrypted_part.1;
		let mut decrypted_part = decrypted_part.0;

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

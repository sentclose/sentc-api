use rustgram::service::IntoResponse;
use rustgram::{Request, Response};
use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, AppRes, JRes, ServerSuccessOutput};
use rustgram_server_util::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};
use sentc_crypto_common::file::{FileNameUpdate, FilePartRegisterOutput, FileRegisterInput, FileRegisterOutput};
use sentc_crypto_common::FileId;
use server_api_common::customer_app::{check_endpoint_with_app_options, check_endpoint_with_req, get_app_data_from_req, Endpoint};
use server_api_common::group::get_group_user_data_from_req;
use server_api_common::user::get_jwt_data_from_param;
use server_dashboard_common::app::{FILE_STORAGE_OWN, FILE_STORAGE_SENTC};

use crate::file_entities::{FileMetaData, FilePartListItem};
use crate::{file_model, file_service, ApiErrorCodes};

pub async fn register_file(mut req: Request) -> JRes<FileRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;
	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::FileRegister)?;

	let user = get_jwt_data_from_param(&req)?;

	let input: FileRegisterInput = bytes_to_json(&body)?;

	let out = file_service::register_file(input, &app.app_data.app_id, &user.id, None).await?;

	echo(out)
}

pub async fn register_file_in_group(mut req: Request) -> JRes<FileRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::FileRegister)?;

	let group_data = get_group_user_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	let input: FileRegisterInput = bytes_to_json(&body)?;

	let out = file_service::register_file(
		input,
		&group_data.group_data.app_id,
		&user.id,
		Some(group_data.group_data.id.clone()),
	)
	.await?;

	echo(out)
}

pub async fn delete_registered_file_part(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::ForceServer)?;

	let file_options = &app.file_options;

	if file_options.file_storage != FILE_STORAGE_OWN {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::FileUploadAllowed,
			"File upload is not allowed",
		));
	}

	let app_id = &app.app_data.app_id;

	let part_id = get_name_param_from_req(&req, "part_id")?;

	file_model::delete_file_part(app_id, part_id).await?;

	echo_success()
}

pub async fn register_file_part(req: Request) -> JRes<FilePartRegisterOutput>
{
	//this fn is called from another storage without a file and we give the file part id back, to name the other file
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::ForceServer)?;

	let file_options = &app.file_options;

	if file_options.file_storage != FILE_STORAGE_OWN {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::FileUploadAllowed,
			"File upload is not allowed",
		));
	}

	let user = get_jwt_data_from_param(&req)?;
	let app_id = &app.app_data.app_id;

	let (file_id, _chunk_size, sequence, end) = check_session(&req, app_id, &user.id).await?;

	let part_id = create_id();

	file_model::save_part(app_id, file_id, part_id.clone(), 0, sequence, end, true).await?;

	echo(FilePartRegisterOutput {
		part_id,
	})
}

pub async fn upload_part(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;
	let file_options = &app.file_options;

	check_endpoint_with_app_options(app, Endpoint::FilePartUpload)?;

	let user = get_jwt_data_from_param(&req)?;
	let app_id = app.app_data.app_id.clone(); //must be owned because req is dropped before save part with app id

	if file_options.file_storage != FILE_STORAGE_SENTC {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::FileUploadAllowed,
			"File upload is not allowed",
		));
	}

	let (file_id, chunk_size, sequence, end) = check_session(&req, &app_id, &user.id).await?;

	//create the id here to upload the right file
	let part_id = create_id();

	let size = rustgram_server_util::file::upload_part(req, &part_id, chunk_size).await?;

	file_model::save_part(&app_id, file_id, part_id, size, sequence, end, false).await?;

	echo_success()
}

async fn check_session(req: &Request, app_id: &str, user_id: &str) -> AppRes<(FileId, usize, i32, bool)>
{
	let params = get_params(req)?;
	let session_id = get_name_param_from_params(params, "session_id")?;
	let sequence = get_name_param_from_params(params, "seq")?;
	let sequence: i32 = sequence.parse().map_err(|_e| {
		ServerCoreError::new_msg(
			400,
			ApiErrorCodes::UnexpectedTime,
			"Parameter sequence has a wrong format",
		)
	})?;
	let end = get_name_param_from_params(params, "end")?;
	let end: bool = end
		.parse()
		.map_err(|_e| ServerCoreError::new_msg(400, ApiErrorCodes::UnexpectedTime, "Parameter end has a wrong format"))?;

	let (file_id, chunk_size) = file_model::check_session(app_id, session_id, user_id).await?;

	Ok((file_id, chunk_size, sequence, end))
}

//__________________________________________________________________________________________________

pub async fn get_file(req: Request) -> JRes<FileMetaData>
{
	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::FileGet)?;

	//use optional user id
	let user_id = match get_jwt_data_from_param(&req) {
		Err(_e) => {
			//only err when jwt was not set -> which is optional here
			//get app id from app data
			None
		},
		Ok(jwt) => Some(jwt.id.as_str()),
	};

	let file_id = get_name_param_from_req(&req, "file_id")?;

	let file = file_service::get_file(&app.app_data.app_id, user_id, file_id, None).await?;

	echo(file)
}

pub async fn get_file_in_group(req: Request) -> JRes<FileMetaData>
{
	check_endpoint_with_req(&req, Endpoint::FileGet)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let file_id = get_name_param_from_req(&req, "file_id")?;
	let app_id = &group_data.group_data.app_id;
	let group_id = &group_data.group_data.id;
	let user_id = &group_data.user_data.user_id;

	let file = file_service::get_file(app_id, Some(user_id), file_id, Some(group_id)).await?;

	echo(file)
}

pub async fn get_parts(req: Request) -> JRes<Vec<FilePartListItem>>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app_data, Endpoint::FileGet)?;

	let params = get_params(&req)?;
	let file_id = get_name_param_from_params(params, "file_id")?;
	let last_sequence = get_name_param_from_params(params, "last_sequence")?;
	let last_sequence: i32 = last_sequence
		.parse()
		.map_err(|_e| ServerCoreError::new_msg(400, ApiErrorCodes::UnexpectedTime, "last fetched sequence is wrong"))?;

	let parts = file_model::get_file_parts(&app_data.app_data.app_id, file_id, last_sequence).await?;

	echo(parts)
}

pub async fn download_part(req: Request) -> Response
{
	match download_part_internally(req).await {
		Ok(res) => res,
		Err(e) => e.into_response(),
	}
}

pub async fn download_part_internally(req: Request) -> AppRes<Response>
{
	check_endpoint_with_req(&req, Endpoint::FilePartDownload)?;

	let part_id = get_name_param_from_req(&req, "part_id")?;

	rustgram_server_util::file::get_part(part_id).await
}

pub async fn update_file_name(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	let app = get_app_data_from_req(&req)?;
	check_endpoint_with_app_options(app, Endpoint::FileRegister)?;

	let user = get_jwt_data_from_param(&req)?;
	let part_id = get_name_param_from_req(&req, "file_id")?;

	let input: FileNameUpdate = bytes_to_json(&body)?;

	file_service::update_file_name(&app.app_data.app_id, &user.id, part_id, input.encrypted_file_name).await?;

	echo_success()
}

//__________________________________________________________________________________________________

pub async fn delete_file(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(app, Endpoint::FileDelete)?;

	let user = get_jwt_data_from_param(&req)?;

	let file_id = get_name_param_from_req(&req, "file_id")?;

	file_service::delete_file(file_id, app.app_data.app_id.as_str(), &user.id, None).await?;

	echo_success()
}

pub async fn delete_file_in_group(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::FileDelete)?;

	let group_data = get_group_user_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	let file_id = get_name_param_from_req(&req, "file_id")?;

	file_service::delete_file(file_id, &group_data.group_data.app_id, &user.id, Some(group_data)).await?;

	echo_success()
}

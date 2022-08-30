use rustgram::{GramHttpErr, Request, Response};
use sentc_crypto_common::file::{FileRegisterInput, FileRegisterOutput};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use server_api_common::app::{FILE_STORAGE_NONE, FILE_STORAGE_SENTC};
use server_core::input_helper::{bytes_to_json, get_raw_body};
use server_core::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params};
use uuid::Uuid;

use crate::customer_app::app_util::{check_endpoint_with_app_options, check_endpoint_with_req, get_app_data_from_req, Endpoint};
use crate::file::file_entities::{FileMetaData, FilePartListItem};
use crate::file::{file_model, file_service};
use crate::group::get_group_user_data_from_req;
use crate::user::jwt::get_jwt_data_from_param;
use crate::util::api_res::{echo, echo_success, ApiErrorCodes, AppRes, HttpErr, JRes};

pub async fn register_file(mut req: Request) -> JRes<FileRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::FileRegister)?;

	let user = get_jwt_data_from_param(&req)?;

	let input: FileRegisterInput = bytes_to_json(&body)?;

	let out = file_service::register_file(input, user.sub.to_string(), user.id.to_string(), None).await?;

	echo(out)
}

pub async fn register_file_in_group(mut req: Request) -> JRes<FileRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::FileRegister)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let input: FileRegisterInput = bytes_to_json(&body)?;

	let out = file_service::register_file(
		input,
		group_data.group_data.app_id.to_string(),
		group_data.user_data.user_id.to_string(),
		Some(group_data.group_data.id.to_string()),
	)
	.await?;

	echo(out)
}

pub async fn upload_part(req: Request) -> JRes<ServerSuccessOutput>
{
	let app = get_app_data_from_req(&req)?;
	let file_options = &app.file_options;

	check_endpoint_with_app_options(&app, Endpoint::FilePartUpload)?;

	let user = get_jwt_data_from_param(&req)?;
	let app_id = user.sub.to_string();

	if file_options.file_storage == FILE_STORAGE_NONE {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::FileUploadAllowed,
			"File upload is not allowed".to_string(),
			None,
		));
	}

	let params = get_params(&req)?;
	let session_id = get_name_param_from_params(&params, "session_id")?;
	let sequence = get_name_param_from_params(&params, "seq")?;
	let sequence: i32 = sequence.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"Parameter sequence has a wrong format".to_string(),
			None,
		)
	})?;
	let end = get_name_param_from_params(&params, "end")?;
	let end: bool = end.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"Parameter end has a wrong format".to_string(),
			None,
		)
	})?;

	let (file_id, chunk_size) = file_model::check_session(app_id.to_string(), session_id.to_string(), user.id.to_string()).await?;

	let part_id = Uuid::new_v4().to_string();

	let (size, extern_storage) = if file_options.file_storage == FILE_STORAGE_SENTC {
		//when customer uses our backend storage
		let size = server_core::file::upload_part(req, part_id.as_str(), chunk_size).await?;

		(size, false)
	} else {
		//use the extern upload for extern storage. We are not saving the size of the part for extern storage.
		//TODO
		(0, true)
	};

	file_model::save_part(app_id, file_id, size, sequence, end, extern_storage).await?;

	echo_success()
}

pub async fn get_file(req: Request) -> JRes<FileMetaData>
{
	check_endpoint_with_req(&req, Endpoint::FileGet)?;

	//use optional user id
	let (app_id, user_id) = match get_jwt_data_from_param(&req) {
		Err(_e) => {
			//only err when jwt was not set -> which is optional here
			//get app id from app data
			let app_data = get_app_data_from_req(&req)?;
			(app_data.app_data.app_id.to_string(), None)
		},
		Ok(jwt) => (jwt.sub.to_string(), Some(jwt.id.to_string())),
	};

	let file_id = get_name_param_from_req(&req, "file_id")?;

	let file = file_service::get_file(app_id, user_id, file_id.to_string(), None).await?;

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

	let file = file_service::get_file(
		app_id.to_string(),
		Some(user_id.to_string()),
		file_id.to_string(),
		Some(group_id.to_string()),
	)
	.await?;

	echo(file)
}

pub async fn get_parts(req: Request) -> JRes<Vec<FilePartListItem>>
{
	let app_data = get_app_data_from_req(&req)?;

	check_endpoint_with_app_options(&app_data, Endpoint::FileGet)?;

	let params = get_params(&req)?;
	let file_id = get_name_param_from_params(&params, "file_id")?;
	let last_sequence = get_name_param_from_params(&params, "last_sequence")?;
	let last_sequence: i32 = last_sequence.parse().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::UnexpectedTime,
			"last fetched sequence is wrong".to_string(),
			None,
		)
	})?;

	let parts = file_model::get_file_parts(
		app_data.app_data.app_id.to_string(),
		file_id.to_string(),
		last_sequence,
	)
	.await?;

	echo(parts)
}

pub async fn download_part(req: Request) -> Response
{
	match download_part_internally(req).await {
		Ok(res) => res,
		Err(e) => e.get_res(),
	}
}

pub async fn download_part_internally(req: Request) -> AppRes<Response>
{
	check_endpoint_with_req(&req, Endpoint::FilePartDownload)?;

	let part_id = get_name_param_from_req(&req, "part_id")?;

	Ok(server_core::file::get_part(part_id).await?)
}

//__________________________________________________________________________________________________

pub async fn delete_file(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::FilePartDownload)?;

	let user = get_jwt_data_from_param(&req)?;

	let file_id = get_name_param_from_req(&req, "file_id")?;

	file_service::delete_file(file_id.to_string(), user.sub.to_string(), user.id.to_string(), None).await?;

	echo_success()
}

pub async fn delete_file_in_group(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::FilePartDownload)?;

	let group_data = get_group_user_data_from_req(&req)?;

	let file_id = get_name_param_from_req(&req, "file_id")?;

	file_service::delete_file(
		file_id.to_string(),
		group_data.group_data.app_id.to_string(),
		group_data.user_data.user_id.to_string(),
		Some(group_data),
	)
	.await?;

	echo_success()
}

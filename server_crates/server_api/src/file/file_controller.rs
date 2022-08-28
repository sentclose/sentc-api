use rustgram::Request;
use sentc_crypto_common::file::{FileRegisterInput, FileRegisterOutput};
use sentc_crypto_common::server_default::ServerSuccessOutput;
use server_core::input_helper::{bytes_to_json, get_raw_body};
use server_core::url_helper::{get_name_param_from_params, get_params};
use uuid::Uuid;

use crate::customer_app::app_util::{check_endpoint_with_req, get_app_data_from_req, Endpoint};
use crate::file::{file_model, file_service};
use crate::user::jwt::get_jwt_data_from_param;
use crate::util::api_res::{echo, echo_success, ApiErrorCodes, HttpErr, JRes};

pub async fn register_file(mut req: Request) -> JRes<FileRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;

	check_endpoint_with_req(&req, Endpoint::FileRegister)?;

	let user = get_jwt_data_from_param(&req)?;

	let input: FileRegisterInput = bytes_to_json(&body)?;

	let out = file_service::register_file(input, user.sub.to_string(), user.id.to_string()).await?;

	echo(out)
}

pub async fn upload_part(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::FilePartUpload)?;

	let user = get_jwt_data_from_param(&req)?;
	let app_id = user.sub.to_string();
	let app = get_app_data_from_req(&req)?;

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

	//TODO get storage options (from app data) and select if our backend or customer backend

	let (file_id, chunk_size) = file_model::check_session(app_id.to_string(), session_id.to_string(), user.id.to_string()).await?;

	let part_id = Uuid::new_v4().to_string();

	let size = server_core::file::upload_part(req, part_id.as_str(), chunk_size).await?;

	file_model::save_part(app_id, file_id, size, sequence, end).await?;

	echo_success()
}

pub async fn get_file(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::FileGet)?;

	echo_success()
}

pub async fn download_part(req: Request) -> JRes<ServerSuccessOutput>
{
	check_endpoint_with_req(&req, Endpoint::FilePartDownload)?;

	echo_success()
}

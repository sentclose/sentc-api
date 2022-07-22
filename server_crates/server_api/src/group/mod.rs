use rustgram::Request;

use crate::core::api_err::HttpErr;

pub(crate) async fn get(_req: Request) -> Result<String, HttpErr>
{
	Ok(format!("group"))
}

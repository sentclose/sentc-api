#[derive(Debug)]
pub enum ApiErrorCodes
{
	NoDbConnection = 10,
	DbExecute = 11,
}

#[derive(Debug)]
pub struct HttpErr
{
	http_status_code: u16,
	api_error_code: ApiErrorCodes,
	msg: &'static str,
	debug_msg: Option<String>,
}

impl HttpErr
{
	pub fn new(http_status_code: u16, api_error_code: ApiErrorCodes, msg: &'static str, debug_msg: Option<String>) -> Self
	{
		Self {
			http_status_code,
			api_error_code,
			msg,
			debug_msg,
		}
	}
}

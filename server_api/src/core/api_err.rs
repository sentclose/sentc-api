#[derive(Debug)]
pub enum ApiErrorCodes
{
	JsonToString = 1,
	JsonParse = 2,

	UnexpectedTimeError = 9,

	NoDbConnection = 10,
	DbQuery = 11,
	DbExecute = 12,

	UserNotFound = 100,
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

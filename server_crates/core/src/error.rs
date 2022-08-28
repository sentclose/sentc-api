#[derive(Debug)]
pub enum CoreErrorCodes
{
	JsonToString,
	JsonParse,

	InputTooBig,

	UnexpectedTime,

	DbQuery,
	DbExecute,
	DbBulkInsert,
	DbTx,
	NoDbConnection,

	EmailMessage,
	EmailSend,

	NoParameter,

	FileLocalOpen,
	FileRemove,
	FileSave,
	FileDownload,
}

#[derive(Debug)]
pub struct CoreError
{
	pub http_status_code: u16,
	pub error_code: CoreErrorCodes,
	pub msg: String,
	pub debug_msg: Option<String>,
}

impl CoreError
{
	pub fn new(http_status_code: u16, error_code: CoreErrorCodes, msg: String, debug_msg: Option<String>) -> Self
	{
		Self {
			http_status_code,
			error_code,
			msg,
			debug_msg,
		}
	}
}

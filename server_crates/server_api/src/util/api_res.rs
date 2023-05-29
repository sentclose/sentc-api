use rustgram_server_util::error::{CoreErrorCodes, ServerErrorCodes};

#[derive(Debug)]
pub enum ApiErrorCodes
{
	Core(CoreErrorCodes),

	PageNotFound,

	JsonToString,
	JsonParse,

	InputTooBig,

	UnexpectedTime,

	NoDbConnection,
	DbQuery,
	DbExecute,
	DbBulkInsert,
	DbTx,

	JwtNotFound,
	JwtWrongFormat,
	JwtValidation,
	JwtCreation,
	JwtKeyCreation,
	JwtKeyNotFound,

	NoParameter,

	EmailSend,
	EmailMessage,

	CustomerWrongAppToken,
	CustomerEmailValidate,
	CustomerNotFound,
	CustomerEmailTokenValid,
	CustomerEmailSyntax,

	UserNotFound,
	UserDeviceDelete,
	UserDeviceNotFound,
	UserKeysNotFound,
	UserExists,
	Login,
	WrongJwtAction,
	AuthKeyFormat,
	SaltError,
	RefreshToken,

	AppTokenNotFound,
	AppTokenWrongFormat,
	AppNotFound,
	AppAction,

	GroupUserNotFound,
	GroupUserRank,
	GroupUserExists,
	GroupNoKeys,
	GroupKeyNotFound,

	GroupTooManyKeys,
	GroupKeySession,
	GroupInviteNotFound,
	GroupOnlyOneAdmin,
	GroupJoinReqNotFound,
	GroupAccess,
	GroupKeyRotationKeysNotFound,
	GroupKeyRotationThread,
	GroupKeyRotationUserEncrypt,
	GroupKeyRotationLimit,
	GroupUserRankUpdate,
	GroupUserKick,
	GroupUserKickRank,
	GroupInviteStop,
	GroupConnectedFromConnected,
	GroupJoinAsConnectedGroup,
	GroupReInviteMemberNotFound,

	KeyNotFound,

	FileSessionNotFound,
	FileSessionExpired,
	FileNotFound,
	FileUploadAllowed,
	FileAccess,

	CaptchaCreate,
	CaptchaNotFound,
	CaptchaTooOld,
	CaptchaWrong,

	ContentItemNotSet,
	ContentItemTooBig,
	ContentCreateItemTooManyCat,

	ContentSearchableItemRefNotSet,
	ContentSearchableItemRefTooBig,
	ContentSearchableNoHashes,
	ContentSearchableTooManyHashes,
	ContentSearchableQueryMissing,
}

impl From<CoreErrorCodes> for ApiErrorCodes
{
	fn from(e: CoreErrorCodes) -> Self
	{
		Self::Core(e)
	}
}

impl ServerErrorCodes for ApiErrorCodes
{
	fn get_int_code(&self) -> u32
	{
		match self {
			ApiErrorCodes::Core(core) => core.get_int_code(),

			ApiErrorCodes::PageNotFound => 404,

			ApiErrorCodes::JsonToString => 10,
			ApiErrorCodes::JsonParse => 11,
			ApiErrorCodes::InputTooBig => 12,
			ApiErrorCodes::UnexpectedTime => 12,

			ApiErrorCodes::NoDbConnection => 20,
			ApiErrorCodes::DbQuery => 21,
			ApiErrorCodes::DbExecute => 22,
			ApiErrorCodes::DbBulkInsert => 23,
			ApiErrorCodes::DbTx => 24,

			ApiErrorCodes::JwtValidation => 30,
			ApiErrorCodes::JwtNotFound => 31,
			ApiErrorCodes::JwtWrongFormat => 32,
			ApiErrorCodes::JwtCreation => 33,
			ApiErrorCodes::JwtKeyCreation => 34,
			ApiErrorCodes::JwtKeyNotFound => 35,

			ApiErrorCodes::NoParameter => 40,

			ApiErrorCodes::EmailSend => 50,
			ApiErrorCodes::EmailMessage => 51,

			ApiErrorCodes::CustomerWrongAppToken => 60,
			ApiErrorCodes::CustomerEmailValidate => 61,
			ApiErrorCodes::CustomerNotFound => 62,
			ApiErrorCodes::CustomerEmailTokenValid => 63,
			ApiErrorCodes::CustomerEmailSyntax => 64,

			ApiErrorCodes::UserNotFound => 100,
			ApiErrorCodes::UserExists => 101,
			ApiErrorCodes::SaltError => 110,
			ApiErrorCodes::AuthKeyFormat => 111,
			ApiErrorCodes::Login => 112,
			ApiErrorCodes::WrongJwtAction => 113,
			ApiErrorCodes::RefreshToken => 114,
			ApiErrorCodes::UserDeviceDelete => 115,
			ApiErrorCodes::UserDeviceNotFound => 116,
			ApiErrorCodes::UserKeysNotFound => 117,

			ApiErrorCodes::AppTokenNotFound => 200,
			ApiErrorCodes::AppTokenWrongFormat => 201,
			ApiErrorCodes::AppNotFound => 202,
			ApiErrorCodes::AppAction => 203,

			ApiErrorCodes::GroupUserNotFound => 300,
			ApiErrorCodes::GroupUserRank => 301,
			ApiErrorCodes::GroupUserExists => 302,
			ApiErrorCodes::GroupNoKeys => 303,
			ApiErrorCodes::GroupKeyNotFound => 304,

			ApiErrorCodes::GroupTooManyKeys => 305,
			ApiErrorCodes::GroupKeySession => 306,
			ApiErrorCodes::GroupInviteNotFound => 307,
			ApiErrorCodes::GroupOnlyOneAdmin => 308,
			ApiErrorCodes::GroupJoinReqNotFound => 309,
			ApiErrorCodes::GroupAccess => 310,
			ApiErrorCodes::GroupKeyRotationKeysNotFound => 311,
			ApiErrorCodes::GroupKeyRotationThread => 312,
			ApiErrorCodes::GroupKeyRotationUserEncrypt => 313,
			ApiErrorCodes::GroupUserRankUpdate => 314,
			ApiErrorCodes::GroupUserKick => 315,
			ApiErrorCodes::GroupUserKickRank => 316,
			ApiErrorCodes::GroupInviteStop => 317,
			ApiErrorCodes::GroupConnectedFromConnected => 318,
			ApiErrorCodes::GroupJoinAsConnectedGroup => 319,
			ApiErrorCodes::GroupReInviteMemberNotFound => 320,
			ApiErrorCodes::GroupKeyRotationLimit => 321,

			ApiErrorCodes::KeyNotFound => 400,

			ApiErrorCodes::FileSessionNotFound => 510,
			ApiErrorCodes::FileSessionExpired => 511,
			ApiErrorCodes::FileNotFound => 512,
			ApiErrorCodes::FileUploadAllowed => 520,
			ApiErrorCodes::FileAccess => 521,

			ApiErrorCodes::CaptchaCreate => 600,
			ApiErrorCodes::CaptchaNotFound => 601,
			ApiErrorCodes::CaptchaTooOld => 602,
			ApiErrorCodes::CaptchaWrong => 603,

			ApiErrorCodes::ContentItemNotSet => 700,
			ApiErrorCodes::ContentItemTooBig => 701,
			ApiErrorCodes::ContentCreateItemTooManyCat => 702,

			ApiErrorCodes::ContentSearchableItemRefNotSet => 800,
			ApiErrorCodes::ContentSearchableItemRefTooBig => 801,
			ApiErrorCodes::ContentSearchableNoHashes => 802,
			ApiErrorCodes::ContentSearchableTooManyHashes => 803,
			ApiErrorCodes::ContentSearchableQueryMissing => 810,
		}
	}
}

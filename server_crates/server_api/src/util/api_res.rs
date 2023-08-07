use rustgram_server_util::error::{CoreErrorCodes, ServerErrorCodes};

#[derive(Debug)]
pub enum ApiErrorCodes
{
	Core(CoreErrorCodes),

	CustomerEmailSyntax,
	CustomerDisable,

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

	AppTokenWrongFormat,

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

	ContentItemNotSet,
	ContentItemTooBig,
	ContentCreateItemTooManyCat,

	ToTpSecretDecode,
	ToTpGet,
	ToTpWrongToken,
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

			ApiErrorCodes::CustomerEmailSyntax => 64,
			ApiErrorCodes::CustomerDisable => 65,

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

			ApiErrorCodes::AppTokenWrongFormat => 201,

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

			ApiErrorCodes::ContentItemNotSet => 700,
			ApiErrorCodes::ContentItemTooBig => 701,
			ApiErrorCodes::ContentCreateItemTooManyCat => 702,

			ApiErrorCodes::ToTpSecretDecode => 900,
			ApiErrorCodes::ToTpGet => 901,
			ApiErrorCodes::ToTpWrongToken => 902,
		}
	}
}

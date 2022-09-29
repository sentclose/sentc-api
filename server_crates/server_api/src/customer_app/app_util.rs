use ring::digest::{Context, SHA256};
use rustgram::Request;

use crate::customer_app::app_entities::AppData;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::AuthWithToken;

pub static HASH_ALG: &'static str = "SHA256";

pub enum Endpoint
{
	ForceServer,

	UserExists,
	UserRegister,
	UserDelete,
	UserPrepLogin,
	UserDoneLogin,
	UserUpdate,
	UserChangePassword,
	UserResetPassword,
	UserPublicData,
	UserRefreshJwt,
	UserDeviceRegister,
	UserDeviceDelete,
	UserDeviceList,
	UserKeyRotation,

	GroupCreate,
	GroupDelete,
	GroupUserDataGet,
	GroupUserKeys,
	GroupUserUpdateCheck,
	GroupInviteStop,

	GroupList,

	GroupKeyRotation,

	GroupInvite,
	GroupAutoInvite,
	GroupAcceptInvite,
	GroupRejectInvite,
	GroupJoinReq,
	GroupAcceptJoinReq,
	GroupRejectJoinReq,

	GroupLeave,
	GroupChangeRank,
	GroupUserDelete,

	KeyRegister,
	KeyGet,

	FileRegister,
	FilePartUpload,
	FileGet,
	FilePartDownload,
}

pub(crate) fn get_app_data_from_req(req: &Request) -> AppRes<&AppData>
{
	//this should always be there because it is checked in the app token mw
	match req.extensions().get::<AppData>() {
		Some(e) => Ok(e),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::AppNotFound,
				"No app found".to_string(),
				None,
			))
		},
	}
}

pub fn hash_token(token: &[u8]) -> AppRes<[u8; 32]>
{
	let mut context = Context::new(&SHA256);
	context.update(token);
	let result = context.finish();

	let hashed_token: [u8; 32] = result.as_ref().try_into().map_err(|_e| {
		HttpErr::new(
			400,
			ApiErrorCodes::AppTokenWrongFormat,
			"Token can't be hashed".to_string(),
			None,
		)
	})?;

	Ok(hashed_token)
}

pub fn hash_token_to_string(token: &[u8]) -> AppRes<String>
{
	let token = hash_token(&token)?;

	Ok(base64::encode(token))
}

pub fn hash_token_from_string_to_string(token: &str) -> AppRes<String>
{
	//the normal token is also encoded as base64 when exporting it to user
	let token = base64::decode(token).map_err(|_e| {
		HttpErr::new(
			401,
			ApiErrorCodes::AppTokenWrongFormat,
			"Token can't be hashed".to_string(),
			None,
		)
	})?;

	hash_token_to_string(&token)
}

pub(crate) fn check_endpoint_with_app_options(app_data: &AppData, endpoint: Endpoint) -> AppRes<()>
{
	let token_used = &app_data.auth_with_token;
	let options = &app_data.options;

	let token_needed = match endpoint {
		Endpoint::UserExists => options.user_exists,
		Endpoint::UserRegister => options.user_register,
		Endpoint::UserDelete => options.user_delete,
		Endpoint::UserPrepLogin => options.user_prepare_login,
		Endpoint::UserDoneLogin => options.user_done_login,
		Endpoint::UserUpdate => options.user_update,
		Endpoint::UserChangePassword => options.user_change_password,
		Endpoint::UserResetPassword => options.user_reset_password,
		Endpoint::UserPublicData => options.user_public_data,
		Endpoint::UserRefreshJwt => options.user_jwt_refresh,
		Endpoint::UserDeviceRegister => options.user_device_register,
		Endpoint::UserDeviceDelete => options.user_device_delete,
		Endpoint::UserDeviceList => options.user_device_list,
		Endpoint::UserKeyRotation => options.user_key_update,

		Endpoint::GroupCreate => options.group_create,
		Endpoint::GroupDelete => options.group_delete,

		Endpoint::GroupList => options.group_list,

		Endpoint::GroupUserDataGet => options.group_get,
		Endpoint::GroupUserKeys => options.group_user_keys,
		Endpoint::GroupUserUpdateCheck => options.group_user_update_check,

		Endpoint::GroupKeyRotation => options.group_key_rotation,
		Endpoint::GroupInvite => options.group_invite,
		Endpoint::GroupAutoInvite => options.group_auto_invite,
		Endpoint::GroupAcceptInvite => options.group_accept_invite,
		Endpoint::GroupRejectInvite => options.group_reject_invite,
		Endpoint::GroupJoinReq => options.group_join_req,
		Endpoint::GroupAcceptJoinReq => options.group_accept_join_req,
		Endpoint::GroupRejectJoinReq => options.group_reject_join_req,

		Endpoint::GroupLeave => options.group_leave,
		Endpoint::GroupChangeRank => options.group_change_rank,
		Endpoint::GroupUserDelete => options.group_user_delete,
		Endpoint::GroupInviteStop => options.group_invite_stop,

		Endpoint::KeyRegister => options.key_register,
		Endpoint::KeyGet => options.key_get,

		Endpoint::FileRegister => options.file_register,
		Endpoint::FilePartUpload => options.file_part_upload,
		Endpoint::FileGet => options.file_get,
		Endpoint::FilePartDownload => options.file_part_download,

		Endpoint::ForceServer => 2,
	};

	let token_needed = match token_needed {
		1 => AuthWithToken::Public,
		2 => AuthWithToken::Secret,
		_ => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::AppAction,
				"No access to this action".to_string(),
				None,
			))
		},
	};

	match (&token_needed, token_used) {
		//both public is ok
		(AuthWithToken::Public, AuthWithToken::Public) => Ok(()),
		//public required but secret is ok because secret > public
		(AuthWithToken::Public, AuthWithToken::Secret) => Ok(()),
		//Both secret is ok
		(AuthWithToken::Secret, AuthWithToken::Secret) => Ok(()),
		//when secret required but public token => err
		(AuthWithToken::Secret, AuthWithToken::Public) => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::AppAction,
				"No access to this action".to_string(),
				None,
			))
		},
	}
}

/**
Check the endpoint with the app options

get the options from req
*/
pub fn check_endpoint_with_req(req: &Request, endpoint: Endpoint) -> AppRes<()>
{
	let data = get_app_data_from_req(req)?;

	check_endpoint_with_app_options(data, endpoint)
}

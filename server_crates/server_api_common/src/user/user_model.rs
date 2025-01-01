use rustgram_server_util::db::{query_first, I64Entity, StringEntity};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::set_params;
use sentc_crypto_common::{AppId, UserId};

pub(super) async fn get_jwt_sign_key(kid: impl Into<String>) -> AppRes<Option<String>>
{
	//language=SQL
	let sql = "SELECT sign_key FROM sentc_app_jwt_keys WHERE id = ?";

	let sign_key: Option<StringEntity> = query_first(sql, set_params!(kid.into())).await?;

	//decrypt the sign key with ear root
	match sign_key {
		Some(sk) => Ok(Some(encrypted_at_rest_root::decrypt(&sk.0).await?)),
		None => Ok(None),
	}
}

pub(super) async fn get_jwt_verify_key(kid: impl Into<String>) -> AppRes<Option<String>>
{
	//language=SQL
	let sql = "SELECT verify_key FROM sentc_app_jwt_keys WHERE id = ?";

	let sign_key: Option<StringEntity> = query_first(sql, set_params!(kid.into())).await?;

	Ok(sign_key.map(|i| i.0))
}

pub(super) async fn get_user_group_id(app_id: impl Into<AppId>, user_id: impl Into<UserId>) -> AppRes<Option<String>>
{
	//language=SQL
	let sql = "SELECT user_group_id FROM sentc_user WHERE app_id = ? AND id = ?";

	let id: Option<StringEntity> = query_first(sql, set_params!(app_id.into(), user_id.into())).await?;

	Ok(id.map(|i| i.0))
}

pub async fn check_user_in_app(app_id: impl Into<AppId>, user_id: impl Into<UserId>) -> AppRes<bool>
{
	//language=SQL
	let sql = "SELECT 1 FROM sentc_user WHERE id = ? AND app_id = ? LIMIT 1";

	let exists: Option<I64Entity> = query_first(sql, set_params!(user_id.into(), app_id.into())).await?;

	match exists {
		Some(_) => Ok(true),
		None => Ok(false),
	}
}

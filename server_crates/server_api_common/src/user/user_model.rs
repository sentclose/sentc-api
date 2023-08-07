use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::db::{exec, query_first, I64Entity, StringEntity};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::{get_time, set_params};
use sentc_crypto_common::{AppId, UserId};

use crate::user::user_entity::CaptchaEntity;

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

pub(super) async fn check_user_in_app(app_id: impl Into<AppId>, user_id: impl Into<UserId>) -> AppRes<bool>
{
	//language=SQL
	let sql = "SELECT 1 FROM sentc_user WHERE id = ? AND app_id = ? LIMIT 1";

	let exists: Option<I64Entity> = query_first(sql, set_params!(user_id.into(), app_id.into())).await?;

	match exists {
		Some(_) => Ok(true),
		None => Ok(false),
	}
}

//__________________________________________________________________________________________________

pub(super) async fn save_captcha_solution(app_id: impl Into<AppId>, solution: String) -> AppRes<String>
{
	let time = get_time()?;
	let captcha_id = create_id();

	//language=SQL
	let sql = "INSERT INTO sentc_captcha (id, app_id, solution, time) VALUES (?,?,?,?)";

	exec(
		sql,
		set_params!(captcha_id.clone(), app_id.into(), solution, time.to_string()),
	)
	.await?;

	Ok(captcha_id)
}

pub(super) async fn get_captcha_solution(id: impl Into<String>, app_id: impl Into<AppId>) -> AppRes<Option<CaptchaEntity>>
{
	//language=SQL
	let sql = "SELECT solution, time FROM sentc_captcha WHERE id = ? AND app_id = ?";

	let out: Option<CaptchaEntity> = query_first(sql, set_params!(id.into(), app_id.into())).await?;

	Ok(out)
}

pub(super) async fn delete_captcha(app_id: impl Into<AppId>, id: String) -> AppRes<()>
{
	//owned id because we got the id from the input

	//language=SQL
	let sql = "DELETE FROM sentc_captcha WHERE id = ? AND app_id = ?";

	exec(sql, set_params!(id, app_id.into())).await?;

	Ok(())
}

use sentc_crypto_common::content::CreateData;
use sentc_crypto_common::ContentId;
use server_core::db::exec;
use server_core::res::AppRes;
use server_core::{get_time, set_params, str_clone, str_get, str_t, u128_get};
use uuid::Uuid;

pub(super) async fn create_content(
	app_id: str_t!(),
	creator_id: str_t!(),
	data: CreateData,
	group_id: Option<str_t!()>,
	user_id: Option<str_t!()>,
) -> AppRes<ContentId>
{
	//own the token in sqlite
	#[cfg(feature = "sqlite")]
	let group_id = match group_id {
		Some(t) => Some(str_get!(t)),
		None => None,
	};

	#[cfg(feature = "sqlite")]
	let user_id = match user_id {
		Some(t) => Some(str_get!(t)),
		None => None,
	};

	let content_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_content (id, app_id, item, time, belongs_to_group, belongs_to_user, creator, category) VALUES (?,?,?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			str_clone!(&content_id),
			str_get!(app_id),
			data.item,
			u128_get!(time),
			group_id,
			user_id,
			str_get!(creator_id),
			data.category
		),
	)
	.await?;

	Ok(content_id)
}

pub(super) async fn delete_content_by_id(app_id: str_t!(), content_id: str_t!()) -> AppRes<()>
{
	//no user access check here because this is only called from a service or backend only

	//language=SQL
	let sql = "DELETE FROM sentc_content WHERE app_id = ? AND id = ?";

	exec(sql, set_params!(str_get!(app_id), str_get!(content_id))).await?;

	Ok(())
}

pub(super) async fn delete_content_by_item(app_id: str_t!(), item: str_t!()) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_content WHERE app_id = ? AND item = ?";

	exec(sql, set_params!(str_get!(app_id), str_get!(item))).await?;

	Ok(())
}

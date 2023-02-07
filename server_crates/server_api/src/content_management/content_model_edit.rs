use sentc_crypto_common::content::CreateData;
use sentc_crypto_common::{AppId, ContentId, GroupId, UserId};
use server_core::db::exec;
use server_core::res::AppRes;
use server_core::{get_time, set_params};
use uuid::Uuid;

pub(super) async fn create_content(
	app_id: impl Into<AppId>,
	creator_id: impl Into<UserId>,
	data: CreateData,
	group_id: Option<GroupId>,
	user_id: Option<UserId>,
) -> AppRes<ContentId>
{
	let content_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_content (id, app_id, item, time, belongs_to_group, belongs_to_user, creator, category) VALUES (?,?,?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			content_id.clone(),
			app_id.into(),
			data.item,
			time.to_string(),
			group_id,
			user_id,
			creator_id.into(),
			data.category
		),
	)
	.await?;

	Ok(content_id)
}

pub(super) async fn delete_content_by_id(app_id: impl Into<AppId>, content_id: impl Into<ContentId>) -> AppRes<()>
{
	//no user access check here because this is only called from a service or backend only

	//language=SQL
	let sql = "DELETE FROM sentc_content WHERE app_id = ? AND id = ?";

	exec(sql, set_params!(app_id.into(), content_id.into())).await?;

	Ok(())
}

pub(super) async fn delete_content_by_item(app_id: impl Into<AppId>, item: impl Into<String>) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_content WHERE app_id = ? AND item = ?";

	exec(sql, set_params!(app_id.into(), item.into())).await?;

	Ok(())
}

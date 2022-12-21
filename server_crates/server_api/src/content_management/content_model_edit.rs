use sentc_crypto_common::content::CreateData;
use sentc_crypto_common::{AppId, ContentId, GroupId, UserId};
use server_core::db::{bulk_insert, exec};
use server_core::{get_time, set_params};
use uuid::Uuid;

use crate::util::api_res::AppRes;

pub(super) async fn create_content(
	app_id: AppId,
	creator_id: UserId,
	data: CreateData,
	group_id: Option<GroupId>,
	user_id: Option<UserId>,
) -> AppRes<ContentId>
{
	let content_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_content (id, app_id, item, time, belongs_to_group, belongs_to_user, creator) VALUES (?,?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			content_id.clone(),
			app_id,
			data.item,
			time.to_string(),
			group_id,
			user_id,
			creator_id
		),
	)
	.await?;

	if !data.cat_ids.is_empty() {
		let coned_content_id = content_id.clone();

		bulk_insert(
			true,
			"sentc_content_category_connect".to_string(),
			vec!["cat_id".to_string(), "content_id".to_string()],
			data.cat_ids,
			move |ob| set_params!(ob.to_string(), coned_content_id.clone()),
		)
		.await?;
	}

	Ok(content_id)
}

pub(super) async fn delete_content_by_id(app_id: AppId, content_id: ContentId) -> AppRes<()>
{
	//no user access check here because this is only called from a service or backend only

	//language=SQL
	let sql = "DELETE FROM sentc_content WHERE app_id = ? AND id = ?";

	exec(sql, set_params!(app_id, content_id)).await?;

	Ok(())
}

pub(super) async fn delete_content_by_item(app_id: AppId, item: String) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_content WHERE app_id = ? AND item = ?";

	exec(sql, set_params!(app_id, item)).await?;

	Ok(())
}

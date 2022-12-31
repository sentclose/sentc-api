use sentc_crypto_common::content::CreateData;
use sentc_crypto_common::{AppId, CategoryId, ContentId, CustomerId, GroupId, UserId};
use server_core::db::{exec, query_string};
use server_core::{get_time, set_params};
use uuid::Uuid;

use crate::content_management::content_entity::ListContentCategoryItem;
use crate::customer_app::app_service;
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
	let sql = "INSERT INTO sentc_content (id, app_id, item, time, belongs_to_group, belongs_to_user, creator, cat_id) VALUES (?,?,?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			content_id.clone(),
			app_id,
			data.item,
			time.to_string(),
			group_id,
			user_id,
			creator_id,
			data.cat_id
		),
	)
	.await?;

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

//__________________________________________________________________________________________________
//category

pub(super) async fn create_cat(customer_id: CustomerId, app_id: AppId, name: String) -> AppRes<CategoryId>
{
	app_service::check_app_exists(customer_id, app_id.clone()).await?;

	let cat_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_content_category (id, name, time, app_id) VALUES (?,?,?,?)";

	exec(sql, set_params!(cat_id.clone(), name, time, app_id)).await?;

	Ok(cat_id)
}

pub(super) async fn delete_cat(customer_id: CustomerId, app_id: AppId, cat_id: CategoryId) -> AppRes<()>
{
	app_service::check_app_exists(customer_id, app_id.clone()).await?;

	//language=SQL
	let sql = "DELETE FROM sentc_content_category WHERE app_id = ? AND id = ?";

	exec(sql, set_params!(app_id, cat_id)).await?;

	Ok(())
}

pub(super) async fn update_cat_name(customer_id: CustomerId, app_id: AppId, cat_id: CategoryId, name: String) -> AppRes<()>
{
	app_service::check_app_exists(customer_id, app_id.clone()).await?;

	//language=SQL
	let sql = "UPDATE sentc_content_category SET name = ? WHERE app_id = ? AND id = ?";

	exec(sql, set_params!(name, app_id, cat_id)).await?;

	Ok(())
}

pub(super) async fn get_cat(
	customer_id: CustomerId,
	app_id: AppId,
	last_fetched_time: u128,
	last_id: CategoryId,
) -> AppRes<Vec<ListContentCategoryItem>>
{
	app_service::check_app_exists(customer_id, app_id.clone()).await?;

	//language=SQL
	let sql = "SELECT id, name, time FROM sentc_content_category WHERE app_id = ?".to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND time <=? AND (time < ? OR (time = ? AND id > ?)) ORDER BY time DESC, id LIMIT 100";

		(
			sql,
			set_params!(
				app_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id
			),
		)
	} else {
		let sql = sql + " ORDER BY time DESC, id LIMIT 100";

		(sql, set_params!(app_id))
	};

	let list: Vec<ListContentCategoryItem> = query_string(sql, params).await?;

	Ok(list)
}

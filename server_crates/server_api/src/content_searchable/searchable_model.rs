use sentc_crypto_common::content_searchable::SearchCreateData;
use sentc_crypto_common::{AppId, CategoryId, ContentId, GroupId, UserId};
use server_core::db::{bulk_insert, exec, query_string};
use server_core::{get_time, set_params};
use uuid::Uuid;

use crate::content_searchable::searchable_entities::ListSearchItem;
use crate::util::api_res::AppRes;

pub(super) async fn create(app_id: AppId, data: SearchCreateData, group_id: Option<GroupId>, user_id: Option<UserId>) -> AppRes<()>
{
	let content_id = Uuid::new_v4().to_string();
	let time = get_time()?;

	//language=SQL
	let sql = r"
INSERT INTO sentc_content_searchable_item 
    (
     id, 
     app_id, 
     belongs_to_group, 
     belongs_to_user, 
     category, 
     item_ref,
     alg,
     key_id,
     time
     ) 
VALUES (?,?,?,?,?,?,?,?,?)";

	exec(
		sql,
		set_params!(
			content_id.clone(),
			app_id,
			group_id,
			user_id,
			data.category,
			data.item_ref,
			data.alg,
			data.key_id,
			time.to_string()
		),
	)
	.await?;

	bulk_insert(
		true,
		"sentc_content_searchable_item_parts".to_string(),
		vec!["item_id".to_string(), "hash".to_string()],
		data.hashes,
		move |ob| {
			//
			set_params!(content_id.clone(), ob.clone())
		},
	)
	.await?;

	Ok(())
}

pub(super) async fn delete(app_id: AppId, item_ref: String) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_content_searchable_item WHERE app_id = ? AND item_ref = ?";

	exec(sql, set_params!(app_id, item_ref)).await?;

	Ok(())
}

pub(super) async fn delete_by_cat(app_id: AppId, item_ref: String, cat_id: CategoryId) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_content_searchable_item WHERE app_id = ? AND item_ref = ? AND category = ?";

	exec(sql, set_params!(app_id, item_ref, cat_id)).await?;

	Ok(())
}

pub(super) async fn search_item_for_group(
	app_id: AppId,
	group_id: GroupId,
	search_hash: String,
	last_fetched_time: u128,
	last_id: ContentId,
	limit: u32,
	cat_id: Option<CategoryId>,
) -> AppRes<Vec<ListSearchItem>>
{
	//access over the group routes
	//access to the hmac key too

	//language=SQL
	let mut sql = r"
SELECT id, item_ref, time 
FROM sentc_content_searchable_item i, sentc_content_searchable_item_parts p 
WHERE 
    hash = ? AND 
    belongs_to_group = ? AND 
    app_id = ? AND 
    item_id = id"
		.to_string();

	if cat_id.is_some() {
		sql += " AND category = ?";
	}

	let params = if last_fetched_time > 0 {
		sql += " AND time <= ? AND (time < ? OR (time = ? AND id > ?)) ORDER BY time DESC, id LIMIT ?";

		if let Some(c_id) = cat_id {
			set_params!(
				search_hash,
				group_id,
				app_id,
				c_id,
				//time params
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id,
				limit
			)
		} else {
			set_params!(
				search_hash,
				group_id,
				app_id,
				//time params
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id,
				limit
			)
		}
	} else {
		sql += " ORDER BY time DESC, id LIMIT ?";

		if let Some(c_id) = cat_id {
			set_params!(search_hash, group_id, app_id, c_id, limit)
		} else {
			set_params!(search_hash, group_id, app_id, limit)
		}
	};

	let list: Vec<ListSearchItem> = query_string(sql, params).await?;

	Ok(list)
}

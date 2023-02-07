use sentc_crypto_common::content_searchable::SearchCreateData;
use server_core::db::{bulk_insert, exec, query_string};
use server_core::res::AppRes;
use server_core::{get_time, set_params, str_clone, str_get, str_t, u128_get};
use uuid::Uuid;

use crate::content_searchable::searchable_entities::ListSearchItem;

pub(super) async fn create(app_id: str_t!(), data: SearchCreateData, group_id: Option<str_t!()>, user_id: Option<str_t!()>) -> AppRes<()>
{
	let content_id = Uuid::new_v4().to_string();
	let time = get_time()?;

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
			str_clone!(&content_id),
			str_get!(app_id),
			group_id,
			user_id,
			data.category,
			data.item_ref,
			data.alg,
			data.key_id,
			u128_get!(time)
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
			set_params!(str_clone!(&content_id), str_clone!(ob))
		},
	)
	.await?;

	Ok(())
}

pub(super) async fn delete(app_id: str_t!(), item_ref: str_t!()) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_content_searchable_item WHERE app_id = ? AND item_ref = ?";

	exec(sql, set_params!(str_get!(app_id), str_get!(item_ref))).await?;

	Ok(())
}

pub(super) async fn delete_by_cat(app_id: str_t!(), item_ref: str_t!(), cat_id: str_t!()) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_content_searchable_item WHERE app_id = ? AND item_ref = ? AND category = ?";

	exec(
		sql,
		set_params!(str_get!(app_id), str_get!(item_ref), str_get!(cat_id)),
	)
	.await?;

	Ok(())
}

pub(super) async fn search_item_for_group(
	app_id: str_t!(),
	group_id: str_t!(),
	search_hash: str_t!(),
	last_fetched_time: u128,
	last_id: str_t!(),
	limit: u32,
	cat_id: Option<str_t!()>,
) -> AppRes<Vec<ListSearchItem>>
{
	//own the token in sqlite
	#[cfg(feature = "sqlite")]
	let cat_id = match cat_id {
		Some(t) => Some(str_get!(t)),
		None => None,
	};

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
				str_get!(search_hash),
				str_get!(group_id),
				str_get!(app_id),
				c_id,
				//time params
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				str_get!(last_id),
				limit
			)
		} else {
			set_params!(
				str_get!(search_hash),
				str_get!(group_id),
				str_get!(app_id),
				//time params
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				str_get!(last_id),
				limit
			)
		}
	} else {
		sql += " ORDER BY time DESC, id LIMIT ?";

		if let Some(c_id) = cat_id {
			set_params!(
				str_get!(search_hash),
				str_get!(group_id),
				str_get!(app_id),
				c_id,
				limit
			)
		} else {
			set_params!(str_get!(search_hash), str_get!(group_id), str_get!(app_id), limit)
		}
	};

	let list: Vec<ListSearchItem> = query_string(sql, params).await?;

	Ok(list)
}

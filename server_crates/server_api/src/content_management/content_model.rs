use sentc_crypto_common::content::CreateData;
use sentc_crypto_common::{AppId, CategoryId, ContentId, GroupId, UserId};
use server_core::db::{bulk_insert, exec, query_string};
use server_core::{get_time, set_params};
use uuid::Uuid;

use crate::content_management::content_entity::ListContentItem;
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

pub(super) async fn get_content(
	app_id: AppId,
	user_id: UserId,
	last_fetched_time: u128,
	last_id: ContentId,
	cat_id: Option<CategoryId>,
) -> AppRes<Vec<ListContentItem>>
{
	//TODO find a way to show for connected group to get the access group (the user id of the connected group)
	// try to use it in the from clause and set the user id (which is the connected group id) as access_from_group

	//language=SQL
	let mut sql = r"
SELECT c.id, item, belongs_to_group, belongs_to_user, creator, time, access_from_group
FROM 
    sentc_content c,
    (
        -- member from parent group
		WITH RECURSIVE children (id) AS ( 
			SELECT g.id from sentc_group g WHERE g.parent IN (SELECT group_id FROM sentc_group_user gu1 WHERE user_id = ?) AND g.app_id = ?
										   
			UNION ALL 
				
			SELECT g1.id FROM children c
					JOIN sentc_group g1 ON c.id = g1.parent AND g1.app_id = ?
		)
		
		SELECT *, null as access_from_group FROM children
		UNION 
		SELECT group_id as id, null as access_from_group FROM sentc_group_user gu WHERE user_id = ? AND type = 0
    ) all_groups
WHERE
    app_id = ? AND (
        belongs_to_user = ? OR 
		creator = ? OR 
        belongs_to_group = all_groups.id OR
		belongs_to_group IN (
			-- member from group as member
			SELECT group_id 
			FROM sentc_group_user gu2 
			WHERE 
			    type = 2 AND 
			    user_id = all_groups.id
		)
    )"
	.to_string();

	if cat_id.is_some() {
		sql += " AND c.id = (SELECT content_id FROM sentc_content_category_connect WHERE cat_id = ?)";
	}

	let params = if last_fetched_time > 0 {
		sql += " AND time <= ? AND (time < ? OR (time = ? AND c.id > ?)) ORDER BY time DESC, c.id LIMIT 100";

		if let Some(c_id) = cat_id {
			set_params!(
				user_id.clone(),
				app_id.clone(),
				app_id.clone(),
				user_id.clone(),
				app_id,
				user_id.clone(),
				user_id,
				c_id,
				//time params
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id
			)
		} else {
			set_params!(
				user_id.clone(),
				app_id.clone(),
				app_id.clone(),
				user_id.clone(),
				app_id,
				user_id.clone(),
				user_id,
				//time params
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id
			)
		}
	} else {
		sql += " ORDER BY time DESC, c.id LIMIT 100";

		if let Some(c_id) = cat_id {
			set_params!(
				user_id.clone(),
				app_id.clone(),
				app_id.clone(),
				user_id.clone(),
				app_id,
				user_id.clone(),
				user_id,
				c_id
			)
		} else {
			set_params!(
				user_id.clone(),
				app_id.clone(),
				app_id.clone(),
				user_id.clone(),
				app_id,
				user_id.clone(),
				user_id,
			)
		}
	};

	let list: Vec<ListContentItem> = query_string(sql, params).await?;

	Ok(list)
}

pub(super) async fn get_content_for_group(
	app_id: AppId,
	group_id: GroupId,
	last_fetched_time: u128,
	last_id: ContentId,
	cat_id: Option<CategoryId>,
) -> AppRes<Vec<ListContentItem>>
{
	//access over the group routes

	//TODO add params

	//TODO check if mysql can "save" queries so the children query dont need to write again

	//language=SQL
	let mut sql = r"
SELECT c.id, item, belongs_to_group, belongs_to_user, creator, time 
FROM sentc_content c 
WHERE app_id = ? AND (
    belongs_to_group = ? OR 
    belongs_to_group IN (
        WITH RECURSIVE children (id) AS ( 
			SELECT g.id from sentc_group g WHERE g.parent = ? AND g.app_id = ?
										   
			UNION ALL 
				
			SELECT g1.id FROM children c
					JOIN sentc_group g1 ON c.id = g1.parent AND g1.app_id = ?
		)
		SELECT * FROM children
    ) OR
    belongs_to_group IN (
		-- member from group as member
		SELECT group_id 
		FROM sentc_group_user gu2 
		WHERE 
			type = 2 AND 
			(
				user_id = ? OR 
				user_id IN (
					WITH RECURSIVE children (id) AS ( 
						SELECT g.id from sentc_group g WHERE g.parent = ? AND g.app_id = ?
													   
						UNION ALL 
							
						SELECT g1.id FROM children c
								JOIN sentc_group g1 ON c.id = g1.parent AND g1.app_id = ?
					)
					SELECT * FROM children
				)
			)
	)
)"
	.to_string();

	if cat_id.is_some() {
		sql += " AND c.id = (SELECT content_id FROM sentc_content_category_connect WHERE cat_id = ?)";
	}

	let params = if last_fetched_time > 0 {
		sql += " AND time <= ? AND (time < ? OR (time = ? AND c.id > ?)) ORDER BY time DESC, c.id LIMIT 100";

		if let Some(c_id) = cat_id {
			set_params!(
				app_id.clone(),
				group_id.clone(),
				group_id,
				app_id.clone(),
				app_id,
				c_id,
				//time params
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id
			)
		} else {
			set_params!(
				app_id.clone(),
				group_id.clone(),
				group_id,
				app_id.clone(),
				app_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id
			)
		}
	} else {
		sql += " ORDER BY time DESC, c.id LIMIT 100";

		if let Some(c_id) = cat_id {
			set_params!(
				app_id.clone(),
				group_id.clone(),
				group_id,
				app_id.clone(),
				app_id,
				c_id
			)
		} else {
			set_params!(app_id.clone(), group_id.clone(), group_id, app_id.clone(), app_id,)
		}
	};

	let list: Vec<ListContentItem> = query_string(sql, params).await?;

	Ok(list)
}

pub(super) async fn get_content_to_user(
	app_id: AppId,
	user_id: UserId,
	last_fetched_time: u128,
	last_id: ContentId,
	cat_id: Option<CategoryId>,
) -> AppRes<Vec<ListContentItem>>
{
	//get content which directly belongs to the actual user
	//language=SQL
	let mut sql = r"
SELECT c.id, item, belongs_to_group, belongs_to_user, creator, time 
FROM sentc_content c 
WHERE belongs_to_user = ? AND app_id = ?"
		.to_string();

	if cat_id.is_some() {
		sql += " AND c.id = (SELECT content_id FROM sentc_content_category_connect WHERE cat_id = ?)";
	}

	let params = if last_fetched_time > 0 {
		sql += " AND time <= ? AND (time < ? OR (time = ? AND c.id > ?)) ORDER BY time DESC, c.id LIMIT 100";
		if let Some(c_id) = cat_id {
			set_params!(
				user_id,
				app_id,
				c_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id
			)
		} else {
			set_params!(
				user_id,
				app_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_id
			)
		}
	} else {
		sql += " ORDER BY time DESC, c.id LIMIT 100";

		if let Some(c_id) = cat_id {
			set_params!(user_id, app_id, c_id)
		} else {
			set_params!(user_id, app_id)
		}
	};

	let list: Vec<ListContentItem> = query_string(sql, params).await?;

	Ok(list)
}

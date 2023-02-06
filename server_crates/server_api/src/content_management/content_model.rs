use server_core::db::{query_first, query_string};
use server_core::{set_params, str_clone, str_get, str_t, u128_get};

use crate::content_management::content_entity::{ContentItemAccess, ListContentItem};
use crate::util::api_res::AppRes;

pub(super) async fn get_content(
	app_id: str_t!(),
	user_id: str_t!(),
	last_fetched_time: u128,
	last_id: str_t!(),
	cat_id: Option<str_t!()>,
) -> AppRes<Vec<ListContentItem>>
{
	let app_id = str_get!(app_id);
	let user_id = str_get!(user_id);

	//own the token in sqlite
	#[cfg(feature = "sqlite")]
	let cat_id = match cat_id {
		Some(t) => Some(str_get!(t)),
		None => None,
	};

	//can't use user_groups in the other cte. i got an mysql syntax error when using it in mysql_async.
	//mysql explain says this are the same because the cte only helps to reduce code length.

	//language=SQL
	let mut sql = r"
WITH RECURSIVE 
group_descendants AS (
    SELECT id, parent FROM sentc_group 
    WHERE id IN (
        SELECT group_id 
        FROM sentc_group_user 
        WHERE user_id = ? AND type = 0
    ) AND app_id = ?
    
    UNION ALL
    
    SELECT g.id, g.parent FROM group_descendants gd
			JOIN sentc_group g ON gd.id = g.parent AND g.app_id = ?
),
group_as_member AS ( 
    SELECT group_id AS group_as_member_id, user_id AS access_from_group 
    FROM sentc_group_user 
    WHERE user_id IN (
        SELECT id FROM group_descendants
    )
)

SELECT con.id, item, belongs_to_group, belongs_to_user, creator, con.time, con.category, group_as_member.access_from_group
FROM 
    sentc_content con 
        LEFT JOIN group_descendants ON belongs_to_group = group_descendants.id
        LEFT JOIN group_as_member ON belongs_to_group = group_as_member.group_as_member_id
WHERE
    app_id = ? AND (
        belongs_to_user = ? OR 
		creator = ? OR 
        belongs_to_group = group_descendants.id OR 
        belongs_to_group = group_as_member.group_as_member_id
    )"
	.to_string();

	if cat_id.is_some() {
		sql += " AND category = ?";
	}

	let params = if last_fetched_time > 0 {
		sql += " AND time <= ? AND (time < ? OR (time = ? AND con.id > ?)) ORDER BY time DESC, con.id LIMIT 100";

		if let Some(c_id) = cat_id {
			set_params!(
				//group params
				str_clone!(user_id),
				str_clone!(app_id),
				str_clone!(app_id),
				//query params
				app_id,
				str_clone!(user_id),
				user_id,
				c_id,
				//time params
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				str_get!(last_id)
			)
		} else {
			set_params!(
				//group params
				str_clone!(user_id),
				str_clone!(app_id),
				str_clone!(app_id),
				//query params
				app_id,
				str_clone!(user_id),
				user_id,
				//time params
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				str_get!(last_id)
			)
		}
	} else {
		sql += " ORDER BY time DESC, con.id LIMIT 100";

		if let Some(c_id) = cat_id {
			set_params!(
				//group params
				str_clone!(user_id),
				str_clone!(app_id),
				str_clone!(app_id),
				//query params
				app_id,
				str_clone!(user_id),
				user_id,
				c_id
			)
		} else {
			set_params!(
				//group params
				str_clone!(user_id),
				str_clone!(app_id),
				str_clone!(app_id),
				//query params
				app_id,
				str_clone!(user_id),
				user_id,
			)
		}
	};

	let list: Vec<ListContentItem> = query_string(sql, params).await?;

	Ok(list)
}

pub(super) async fn get_content_for_group(
	app_id: str_t!(),
	group_id: str_t!(),
	last_fetched_time: u128,
	last_id: str_t!(),
	cat_id: Option<str_t!()>,
) -> AppRes<Vec<ListContentItem>>
{
	let app_id = str_get!(app_id);
	let group_id = str_get!(group_id);

	//own the token in sqlite
	#[cfg(feature = "sqlite")]
	let cat_id = match cat_id {
		Some(t) => Some(str_get!(t)),
		None => None,
	};

	//access over the group routes

	//language=SQL
	let mut sql = r"
WITH RECURSIVE 
children AS ( 
	-- get all children group of the groups where the user is direct member
	SELECT g.id as children_id from sentc_group g WHERE g.parent = ? AND g.app_id = ?
								   
	UNION ALL 
		
	SELECT g1.id as children_id FROM children c
			JOIN sentc_group g1 ON c.children_id = g1.parent AND g1.app_id = ?
),
group_as_member AS ( 
	-- get all groups, where the groups where the user got access, are in
	SELECT gu2.group_id as group_as_member_id, gu2.user_id as access_from_group
	FROM sentc_group_user gu2 
	WHERE 
		gu2.type = 2 AND (
		    user_id = ? OR 
		    user_id IN (SELECT * FROM children)
		)
)

SELECT con.id, item, belongs_to_group, belongs_to_user, creator, con.time, con.category, group_as_member.access_from_group 
FROM sentc_content con 
    LEFT JOIN children ON belongs_to_group = children.children_id 
    LEFT JOIN group_as_member ON belongs_to_group = group_as_member.group_as_member_id
WHERE 
    app_id = ? AND (
        belongs_to_group = ? OR 
        belongs_to_group = children.children_id OR 
        belongs_to_group = group_as_member.group_as_member_id
    )"
	.to_string();

	if cat_id.is_some() {
		sql += " AND category = ?";
	}

	let params = if last_fetched_time > 0 {
		sql += " AND time <= ? AND (time < ? OR (time = ? AND con.id > ?)) ORDER BY time DESC, con.id LIMIT 100";

		if let Some(c_id) = cat_id {
			set_params!(
				//children params
				str_clone!(group_id),
				str_clone!(app_id),
				str_clone!(app_id),
				//group as member params
				str_clone!(group_id),
				//query params
				app_id,
				group_id,
				c_id,
				//time params
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				str_get!(last_id)
			)
		} else {
			set_params!(
				//children params
				str_clone!(group_id),
				str_clone!(app_id),
				str_clone!(app_id),
				//group as member params
				str_clone!(group_id),
				//query params
				app_id,
				group_id,
				//time params
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				str_get!(last_id)
			)
		}
	} else {
		sql += " ORDER BY time DESC, con.id LIMIT 100";

		if let Some(c_id) = cat_id {
			set_params!(
				//children params
				str_clone!(group_id),
				str_clone!(app_id),
				str_clone!(app_id),
				//group as member params
				str_clone!(group_id),
				//query params
				app_id,
				group_id,
				c_id
			)
		} else {
			set_params!(
				//children params
				str_clone!(group_id),
				str_clone!(app_id),
				str_clone!(app_id),
				//group as member params
				str_clone!(group_id),
				//query params
				app_id,
				group_id
			)
		}
	};

	let list: Vec<ListContentItem> = query_string(sql, params).await?;

	Ok(list)
}

pub(super) async fn get_content_to_user(
	app_id: str_t!(),
	user_id: str_t!(),
	last_fetched_time: u128,
	last_id: str_t!(),
	cat_id: Option<str_t!()>,
) -> AppRes<Vec<ListContentItem>>
{
	//get content which directly belongs to the actual user
	//language=SQL
	let mut sql = r"
SELECT c.id, item, belongs_to_group, belongs_to_user, creator, time, category, null AS access_from_group
FROM sentc_content c 
WHERE belongs_to_user = ? AND app_id = ?"
		.to_string();

	if cat_id.is_some() {
		sql += " AND category = ?";
	}

	//own the token in sqlite
	#[cfg(feature = "sqlite")]
	let cat_id = match cat_id {
		Some(t) => Some(str_get!(t)),
		None => None,
	};

	let params = if last_fetched_time > 0 {
		sql += " AND time <= ? AND (time < ? OR (time = ? AND c.id > ?)) ORDER BY time DESC, c.id LIMIT 100";
		if let Some(c_id) = cat_id {
			set_params!(
				str_get!(user_id),
				str_get!(app_id),
				c_id,
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				str_get!(last_id)
			)
		} else {
			set_params!(
				str_get!(user_id),
				str_get!(app_id),
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				u128_get!(last_fetched_time),
				str_get!(last_id)
			)
		}
	} else {
		sql += " ORDER BY time DESC, c.id LIMIT 100";

		if let Some(c_id) = cat_id {
			set_params!(str_get!(user_id), str_get!(app_id), c_id)
		} else {
			set_params!(str_get!(user_id), str_get!(app_id))
		}
	};

	let list: Vec<ListContentItem> = query_string(sql, params).await?;

	Ok(list)
}

pub(super) async fn check_access_to_content_by_item(app_id: str_t!(), user_id: str_t!(), item: str_t!()) -> AppRes<ContentItemAccess>
{
	/*
	   Do not return from which group (direct or children) the user got access or from belongs to user / is creator.
	   This is because if the content was placed in a group, the group must be loaded anyways
	   Only access from group is set to know from which connected group the user should load the groups
	*/

	//language=SQL
	let sql = r"
WITH RECURSIVE 
group_descendants AS (
    SELECT id, parent FROM sentc_group 
    WHERE id IN (
        SELECT group_id 
        FROM sentc_group_user 
        WHERE user_id = ? AND type = 0
    ) AND app_id = ?
    
    UNION ALL
    
    SELECT g.id, g.parent FROM group_descendants gd
			JOIN sentc_group g ON gd.id = g.parent AND g.app_id = ?
),
group_as_member AS ( 
    SELECT group_id AS group_as_member_id, user_id AS access_from_group 
    FROM sentc_group_user 
    WHERE user_id IN (
        SELECT id FROM group_descendants
    )
)

SELECT true as access, group_as_member.access_from_group 
FROM sentc_content con
    	LEFT JOIN group_descendants ON belongs_to_group = group_descendants.id
        LEFT JOIN group_as_member ON belongs_to_group = group_as_member.group_as_member_id
WHERE app_id = ? AND 
      item = ? AND (
    	belongs_to_user = ? OR 
		creator = ? OR 
        belongs_to_group = group_descendants.id OR 
        belongs_to_group = group_as_member.group_as_member_id
	)
LIMIT 1
";

	let app_id = str_get!(app_id);
	let user_id = str_get!(user_id);

	let out: Option<ContentItemAccess> = query_first(
		sql,
		set_params!(
			//group params
			str_clone!(user_id),
			str_clone!(app_id),
			str_clone!(app_id),
			//query params
			app_id,
			str_get!(item),
			str_clone!(user_id),
			user_id
		),
	)
	.await?;

	if let Some(o) = out {
		Ok(o)
	} else {
		Ok(ContentItemAccess {
			access: false,
			access_from_group: None,
		})
	}
}

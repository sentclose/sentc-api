use rustgram_server_util::db::{bulk_insert, query_string};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::{get_time, set_params};
use sentc_crypto_common::{AppId, GroupId, SymKeyId};

use crate::ext::group::group_ext_entities::FetchedExt;

pub(super) async fn create(
	app_id: impl Into<AppId>,
	group_id: impl Into<GroupId>,
	encrypted_key_id: impl Into<SymKeyId>,
	ext_data: Vec<(String, String, String)>,
) -> AppRes<()>
{
	let app_id = app_id.into();
	let group_id = group_id.into();
	let encrypted_key_id = encrypted_key_id.into();

	let time = get_time()?;

	//language=SQL
	//let sql = "INSERT INTO sentc_group_ext (id, app_id, group_id, ext_name, ext_data, encrypted_key_id, time) VALUES (?,?,?,?,?,?,?)";

	bulk_insert(
		true,
		"sentc_group_ext",
		&["id", "app_id", "group_id", "ext_name", "ext_data", "encrypted_key_id", "time"],
		ext_data,
		move |o| {
			let ext_name = o.0;
			let id = o.1;
			let ext_data = o.2;

			set_params!(
				id,
				app_id.clone(),
				group_id.clone(),
				ext_name,
				ext_data,
				encrypted_key_id.clone(),
				time.to_string()
			)
		},
	)
	.await?;

	Ok(())
}

pub(super) async fn get_ext(
	app_id: impl Into<AppId>,
	group_id: impl Into<GroupId>,
	last_fetched_time: u128,
	last_ext_id: impl Into<SymKeyId>,
) -> AppRes<Vec<FetchedExt>>
{
	//get all ext data for this group. an ext can be there multiple times but with different data but the same name

	//language=SQL
	let sql = r"
SELECT id, ext_name, ext_data, encrypted_key_id, time 
FROM sentc_group_ext 
WHERE group_id = ? AND app_id = ?
"
	.to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND time <= ? AND (time < ? OR (time = ? AND id > ?)) ORDER BY time DESC, id LIMIT 50";
		(
			sql,
			set_params!(
				group_id.into(),
				app_id.into(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_ext_id.into()
			),
		)
	} else {
		let sql = sql + " ORDER BY time DESC, id LIMIT 50";
		(sql, set_params!(group_id.into(), app_id.into()))
	};

	let data: Vec<FetchedExt> = query_string(sql, params).await?;

	Ok(data)
}

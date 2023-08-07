use rustgram_server_util::db::{exec, exec_string, get_in, TupleEntity};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::{get_time, set_params, set_params_vec};
use sentc_crypto_common::{AppId, CustomerId, GroupId};

use crate::file::{FILE_BELONGS_TO_TYPE_GROUP, FILE_STATUS_TO_DELETE};

pub(super) async fn delete_files_for_customer_group(group_id: impl Into<GroupId>) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = r"
UPDATE 
    sentc_file 
SET 
    status = ?, 
    delete_at = ? 
WHERE 
    app_id IN (
    SELECT 
        sentc_app.id 
    FROM sentc_app 
    WHERE owner_id = ? AND 
          owner_type = 1
    )";

	exec(
		sql,
		set_params!(FILE_STATUS_TO_DELETE, time.to_string(), group_id.into()),
	)
	.await?;

	Ok(())
}

pub(super) async fn delete_files_for_customer(customer_id: impl Into<CustomerId>) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = r"
UPDATE 
    sentc_file 
SET 
    status = ?, 
    delete_at = ? 
WHERE 
    app_id IN (
    SELECT 
        sentc_app.id 
    FROM sentc_app 
    WHERE owner_id = ? AND 
          owner_type = 0
    )";

	exec(
		sql,
		set_params!(FILE_STATUS_TO_DELETE, time.to_string(), customer_id.into()),
	)
	.await?;

	Ok(())
}

pub(super) async fn delete_files_for_app(app_id: impl Into<AppId>) -> AppRes<()>
{
	let time = get_time()?;

	//language=SQL
	let sql = "UPDATE sentc_file SET status = ?, delete_at = ? WHERE app_id = ?";

	exec(
		sql,
		set_params!(FILE_STATUS_TO_DELETE, time.to_string(), app_id.into()),
	)
	.await?;

	Ok(())
}

pub(super) async fn delete_files_for_group(app_id: impl Into<AppId>, group_id: impl Into<GroupId>, children: Vec<String>) -> AppRes<()>
{
	let app_id = app_id.into();
	let time = get_time()?;

	//language=SQL
	let sql = r"
UPDATE 
    sentc_file 
SET
    status = ?, 
    delete_at = ? 
WHERE 
    app_id = ? AND 
    belongs_to_type = ? AND  
	belongs_to = ?";

	exec(
		sql,
		set_params!(
			FILE_STATUS_TO_DELETE,
			time.to_string(),
			app_id.clone(),
			FILE_BELONGS_TO_TYPE_GROUP,
			group_id.into(),
		),
	)
	.await?;

	//update children, can't use mysql recursion here, because it says the rec table doesn't exist
	if !children.is_empty() {
		let get_in = get_in(&children);

		//language=SQLx
		let sql = format!(
			"UPDATE sentc_file SET status = ?, delete_at = ? WHERE app_id = ? AND belongs_to_type = ? AND belongs_to IN ({})",
			get_in
		);

		let mut exec_vec = Vec::with_capacity(children.len() + 4);

		exec_vec.push(TupleEntity(FILE_STATUS_TO_DELETE.to_string()));
		exec_vec.push(TupleEntity(time.to_string()));
		exec_vec.push(TupleEntity(app_id));
		exec_vec.push(TupleEntity(FILE_BELONGS_TO_TYPE_GROUP.to_string()));

		for child in children {
			exec_vec.push(TupleEntity(child));
		}

		exec_string(sql, set_params_vec!(exec_vec)).await?;
	}

	Ok(())
}

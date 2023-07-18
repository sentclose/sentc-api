use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::{AppId, GroupId, SymKeyId};
use sentc_sdk_ext_common::sortable::{SortableCreateData, SortableKeyData};
use sentc_sdk_ext_common::{GroupExtCreate, GroupExtGet};

use crate::ext::group::group_ext_model;
use crate::util::api_res::ApiErrorCodes;

pub async fn create(
	app_id: impl Into<AppId>,
	group_id: impl Into<GroupId>,
	encrypted_key_id: impl Into<SymKeyId>,
	ext: Vec<GroupExtCreate>,
) -> AppRes<()>
{
	if ext.len() > 5 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::ExtTooMany,
			"Too many extensions. Max is 5",
		));
	}

	let mut ext_data = Vec::with_capacity(ext.len());

	//loop over all ext and sort it
	for ex in ext {
		match ex {
			GroupExtCreate::Sortable(e) => {
				ext_data.push((
					"sortable".to_string(),
					create_id(),
					serde_json::to_string(&e)
						.map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::JsonToString, "Can't parse the group input"))?,
				))
			},
		}
	}

	group_ext_model::create(app_id, group_id, encrypted_key_id, ext_data).await?;

	Ok(())
}

pub async fn get_ext(
	app_id: impl Into<AppId>,
	group_id: impl Into<GroupId>,
	last_fetched_time: u128,
	last_ext_id: impl Into<SymKeyId>,
) -> AppRes<Vec<GroupExtGet>>
{
	let ext = group_ext_model::get_ext(app_id, group_id, last_fetched_time, last_ext_id).await?;

	//Sort the ext

	let mut out_ext = Vec::with_capacity(ext.len());

	for e in ext {
		out_ext.push(match e.ext_name.as_str() {
			"sortable-crypto" => {
				//this is stored as string in the db
				let insert_data: SortableCreateData =
					serde_json::from_str(&e.ext_data).map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::JsonParse, "Can't parse ext"))?;

				GroupExtGet::Sortable(SortableKeyData {
					key_id: e.id,
					encrypted_sortable_key: insert_data.encrypted_key,
					encrypted_sortable_alg: insert_data.key_alg,
					encrypted_sortable_encryption_key_id: e.encrypted_key_id,
					time: e.time,
				})
			},
			_ => continue,
		});
	}

	Ok(out_ext)
}

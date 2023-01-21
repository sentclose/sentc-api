use sentc_crypto_common::ContentId;
use serde::Serialize;

#[derive(Serialize)]
#[cfg_attr(feature = "mysql", derive(server_core::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(server_core::Sqlite))]
pub struct ListSearchItem
{
	pub id: ContentId,
	pub item_ref: String,
	pub time: u128,
}

impl Into<sentc_crypto_common::content_searchable::ListSearchItem> for ListSearchItem
{
	fn into(self) -> sentc_crypto_common::content_searchable::ListSearchItem
	{
		sentc_crypto_common::content_searchable::ListSearchItem {
			id: self.id,
			item_ref: self.item_ref,
			time: self.time,
		}
	}
}

//__________________________________________________________________________________________________

use sentc_crypto_common::{CategoryId, ContentId, GroupId, UserId};
use serde::Serialize;

#[derive(Serialize)]
#[cfg_attr(feature = "mysql", derive(server_core::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(server_core::Sqlite))]
pub struct ListContentItem
{
	pub id: ContentId,
	pub item: String,
	pub belongs_to_group: Option<GroupId>,
	pub belongs_to_user: Option<UserId>,
	pub creator: UserId,
	pub time: u128,
	pub access_from_group: Option<GroupId>,
}

impl Into<sentc_crypto_common::content::ListContentItem> for ListContentItem
{
	fn into(self) -> sentc_crypto_common::content::ListContentItem
	{
		sentc_crypto_common::content::ListContentItem {
			id: self.id,
			item: self.item,
			belongs_to_group: self.belongs_to_group,
			belongs_to_user: self.belongs_to_user,
			creator: self.creator,
			time: self.time,
			access_from_group: self.access_from_group,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize)]
#[cfg_attr(feature = "mysql", derive(server_core::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(server_core::Sqlite))]
pub struct ContentItemAccess
{
	pub access: bool,
	pub access_from_group: Option<GroupId>,
}

impl Into<sentc_crypto_common::content::ContentItemAccess> for ContentItemAccess
{
	fn into(self) -> sentc_crypto_common::content::ContentItemAccess
	{
		sentc_crypto_common::content::ContentItemAccess {
			access: self.access,
			access_from_group: self.access_from_group,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize)]
#[cfg_attr(feature = "mysql", derive(server_core::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(server_core::Sqlite))]
pub struct ListContentCategoryItem
{
	pub cat_id: CategoryId,
	pub name: String,
	pub time: u128,
}

impl Into<sentc_crypto_common::content::ListContentCategoryItem> for ListContentCategoryItem
{
	fn into(self) -> sentc_crypto_common::content::ListContentCategoryItem
	{
		sentc_crypto_common::content::ListContentCategoryItem {
			cat_id: self.cat_id,
			name: self.name,
			time: self.time,
		}
	}
}

//__________________________________________________________________________________________________

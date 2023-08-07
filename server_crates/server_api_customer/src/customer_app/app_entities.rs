use rustgram_server_util::DB;
use sentc_crypto_common::{AppId, CustomerId, GroupId};

pub const CUSTOMER_OWNER_TYPE_USER: i32 = 0;
pub const CUSTOMER_OWNER_TYPE_GROUP: i32 = 1;

//__________________________________________________________________________________________________

#[derive(DB)]
pub struct AppCustomerAccess
{
	pub app_id: AppId,
	pub owner_id: CustomerId,
	pub owner_type: i32,
	pub hashed_secret_token: String,
	pub hashed_public_token: String,
	pub hash_alg: String,
	pub group_id: GroupId,
	pub rank: i32,
}

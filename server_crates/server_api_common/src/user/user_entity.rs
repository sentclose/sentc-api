use sentc_crypto_common::{DeviceId, GroupId, UserId};
use serde::{Deserialize, Serialize};

//__________________________________________________________________________________________________
//Jwt

#[derive(Serialize, Deserialize)]
pub struct UserJwtEntity
{
	pub id: UserId,
	pub device_id: DeviceId,
	pub group_id: GroupId,
	pub fresh: bool,
}

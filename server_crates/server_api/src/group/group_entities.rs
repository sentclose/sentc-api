use rustgram_server_util::DB;
use sentc_crypto_common::group::{GroupInviteReqList, GroupJoinReqList, GroupKeyServerOutput, GroupUserAccessBy, KeyRotationInput};
use sentc_crypto_common::{EncryptionKeyPairId, GroupId, SignKeyPairId, SymKeyId, UserId};
use serde::{Deserialize, Serialize};

pub type GroupNewUserType = u16;

/**
invite (keys needed)
*/
pub const GROUP_INVITE_TYPE_INVITE_REQ: GroupNewUserType = 0;

/**
join req (no keys needed)
*/
pub const GROUP_INVITE_TYPE_JOIN_REQ: GroupNewUserType = 1;

//__________________________________________________________________________________________________

/**
Gets build by the controller
*/
#[derive(Serialize)]
pub struct GroupServerData
{
	pub group_id: GroupId,
	pub parent_group_id: Option<GroupId>,
	pub keys: Vec<GroupUserKeys>,
	pub hmac_keys: Vec<GroupHmacData>,
	pub sortable_keys: Vec<GroupSortableData>,
	pub key_update: bool,
	pub rank: i32,
	pub created_time: u128,
	pub joined_time: u128,
	pub access_by: GroupUserAccessBy,
	pub is_connected_group: bool,
}

impl Into<sentc_crypto_common::group::GroupServerData> for GroupServerData
{
	fn into(self) -> sentc_crypto_common::group::GroupServerData
	{
		sentc_crypto_common::group::GroupServerData {
			group_id: self.group_id,
			parent_group_id: self.parent_group_id,
			keys: self.keys.into_iter().map(|k| k.into()).collect(),
			hmac_keys: self.hmac_keys.into_iter().map(|k| k.into()).collect(),
			sortable_keys: self.sortable_keys.into_iter().map(|k| k.into()).collect(),
			key_update: self.key_update,
			rank: self.rank,
			created_time: self.created_time,
			joined_time: self.joined_time,
			access_by: self.access_by,
			is_connected_group: self.is_connected_group,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, DB)]
pub struct GroupHmacData
{
	pub id: SymKeyId,
	pub encrypted_hmac_key: String,
	pub encrypted_hmac_alg: String,
	pub encrypted_hmac_encryption_key_id: SymKeyId,
	pub time: u128,
}

impl Into<sentc_crypto_common::group::GroupHmacData> for GroupHmacData
{
	fn into(self) -> sentc_crypto_common::group::GroupHmacData
	{
		sentc_crypto_common::group::GroupHmacData {
			id: self.id,
			encrypted_hmac_key: self.encrypted_hmac_key,
			encrypted_hmac_alg: self.encrypted_hmac_alg,
			encrypted_hmac_encryption_key_id: self.encrypted_hmac_encryption_key_id,
			time: self.time,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, Deserialize, DB)]
pub struct GroupSortableData
{
	pub id: SymKeyId,
	pub encrypted_sortable_key: String,
	pub encrypted_sortable_alg: String,
	pub encrypted_sortable_encryption_key_id: SymKeyId,
	pub time: u128,
}

impl Into<sentc_crypto_common::group::GroupSortableData> for GroupSortableData
{
	fn into(self) -> sentc_crypto_common::group::GroupSortableData
	{
		sentc_crypto_common::group::GroupSortableData {
			id: self.id,
			encrypted_sortable_key: self.encrypted_sortable_key,
			encrypted_sortable_alg: self.encrypted_sortable_alg,
			encrypted_sortable_encryption_key_id: self.encrypted_sortable_encryption_key_id,
			time: self.time,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, DB)]
pub struct GroupUserKeys
{
	pub key_pair_id: EncryptionKeyPairId,
	pub group_key_id: SymKeyId,
	pub encrypted_group_key: String,
	pub group_key_alg: String,
	pub encrypted_private_group_key: String,
	pub public_group_key: String,
	pub keypair_encrypt_alg: String,
	pub user_public_key_id: EncryptionKeyPairId,
	pub time: u128,

	//if user signed the new group key
	#[serde(skip_serializing_if = "Option::is_none")]
	pub signed_by_user_id: Option<UserId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub signed_by_user_sign_key_id: Option<SignKeyPairId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub group_key_sig: Option<String>,

	//these keys are only set for user group
	#[serde(skip_serializing_if = "Option::is_none")]
	pub encrypted_sign_key: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub verify_key: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub keypair_sign_alg: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub keypair_sign_id: Option<SignKeyPairId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub public_key_sig: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub public_key_sig_key_id: Option<String>,
}

impl Into<GroupKeyServerOutput> for GroupUserKeys
{
	fn into(self) -> GroupKeyServerOutput
	{
		GroupKeyServerOutput {
			encrypted_group_key: self.encrypted_group_key,
			group_key_alg: self.group_key_alg,
			group_key_id: self.group_key_id,
			encrypted_private_group_key: self.encrypted_private_group_key,
			public_group_key: self.public_group_key,
			keypair_encrypt_alg: self.keypair_encrypt_alg,
			key_pair_id: self.key_pair_id,
			user_public_key_id: self.user_public_key_id,
			time: self.time,
			signed_by_user_id: self.signed_by_user_id,
			signed_by_user_sign_key_id: self.signed_by_user_sign_key_id,
			group_key_sig: self.group_key_sig,
			encrypted_sign_key: self.encrypted_sign_key,
			verify_key: self.verify_key,
			keypair_sign_alg: self.keypair_sign_alg,
			keypair_sign_id: self.keypair_sign_id,
			public_key_sig: self.public_key_sig,
			public_key_sig_key_id: self.public_key_sig_key_id,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, DB)]
pub struct GroupJoinReq
{
	pub user_id: UserId,
	pub time: u128,
	pub user_type: i32,
}

impl Into<GroupJoinReqList> for GroupJoinReq
{
	fn into(self) -> GroupJoinReqList
	{
		GroupJoinReqList {
			user_id: self.user_id,
			time: self.time,
			user_type: self.user_type,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, DB)]
pub struct GroupInviteReq
{
	pub group_id: GroupId,
	pub time: u128,
}

impl Into<GroupInviteReqList> for GroupInviteReq
{
	fn into(self) -> GroupInviteReqList
	{
		GroupInviteReqList {
			group_id: self.group_id,
			time: self.time,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, DB)]
pub struct GroupKeyUpdate
{
	pub new_group_key_id: SymKeyId,
	pub error: Option<String>,
	pub encrypted_ephemeral_key_by_group_key_and_public_key: String,
	pub encrypted_eph_key_key_id: String,
	pub encrypted_group_key_by_ephemeral: EncryptionKeyPairId,
	pub previous_group_key_id: SymKeyId,
	pub ephemeral_alg: String,
	pub time: u128,
}

impl Into<KeyRotationInput> for GroupKeyUpdate
{
	fn into(self) -> KeyRotationInput
	{
		KeyRotationInput {
			error: self.error,
			encrypted_ephemeral_key_by_group_key_and_public_key: self.encrypted_ephemeral_key_by_group_key_and_public_key,
			encrypted_group_key_by_ephemeral: self.encrypted_group_key_by_ephemeral,
			ephemeral_alg: self.ephemeral_alg,
			previous_group_key_id: self.previous_group_key_id,
			encrypted_eph_key_key_id: self.encrypted_eph_key_key_id,
			time: self.time,
			new_group_key_id: self.new_group_key_id,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(DB)]
pub struct KeyRotationWorkerKey
{
	pub ephemeral_alg: String,
	pub encrypted_ephemeral_key: String,
}

//__________________________________________________________________________________________________

/**
Output after key rotation for each user
*/
pub struct UserEphKeyOut
{
	pub user_id: UserId,
	pub encrypted_ephemeral_key: String,
	pub encrypted_eph_key_key_id: EncryptionKeyPairId,
	pub rotation_err: Option<String>,
}

//__________________________________________________________________________________________________

#[derive(DB)]
pub struct UserGroupPublicKeyData
{
	pub user_id: UserId,
	pub public_key_id: EncryptionKeyPairId,
	pub public_key: String,
	pub public_key_alg: String,
	pub time: u128,
}

//__________________________________________________________________________________________________

#[derive(Serialize, DB)]
pub struct GroupUserListItem
{
	pub user_id: UserId,
	pub rank: i32,
	pub joined_time: u128,
	pub user_type: i32,
}

impl Into<sentc_crypto_common::group::GroupUserListItem> for GroupUserListItem
{
	fn into(self) -> sentc_crypto_common::group::GroupUserListItem
	{
		sentc_crypto_common::group::GroupUserListItem {
			user_id: self.user_id,
			rank: self.rank,
			joined_time: self.joined_time,
			user_type: self.user_type,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, DB)]
pub struct ListGroups
{
	pub group_id: GroupId,
	pub time: u128,
	pub joined_time: u128,
	pub rank: i32,
	pub parent: Option<GroupId>,
}

impl Into<sentc_crypto_common::group::ListGroups> for ListGroups
{
	fn into(self) -> sentc_crypto_common::group::ListGroups
	{
		sentc_crypto_common::group::ListGroups {
			group_id: self.group_id,
			time: self.time,
			joined_time: self.joined_time,
			rank: self.rank,
			parent: self.parent,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Serialize, DB)]
pub struct GroupChildrenList
{
	pub group_id: GroupId,
	pub time: u128,
	pub parent: Option<GroupId>,
}

impl Into<sentc_crypto_common::group::GroupChildrenList> for GroupChildrenList
{
	fn into(self) -> sentc_crypto_common::group::GroupChildrenList
	{
		sentc_crypto_common::group::GroupChildrenList {
			group_id: self.group_id,
			time: self.time,
			parent: self.parent,
		}
	}
}

//__________________________________________________________________________________________________

#[derive(DB)]
pub struct GroupUserInvitesAndJoinReq
{
	pub user_type: i32,
	pub new_user_rank: i32,
}

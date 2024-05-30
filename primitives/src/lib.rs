#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::parameter_types;
use scale_info::{prelude::vec::Vec, TypeInfo};
use serde::{Deserialize, Serialize};
use sp_core::{crypto::KeyTypeId, hash::H160, H256};
use sp_runtime::{AccountId32, Perquintill, RuntimeDebug};

pub mod traits;

parameter_types! {
	pub MaxHostLen: u8 = 255;
	pub MaxDomainLen: u8 = 255;
}

pub const MILLICENTS: u128 = 100_000;
pub const CENTS: u128 = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: u128 = 100 * CENTS;
pub type ClusterId = H160;
pub type DdcEra = u32;
pub type BucketId = u64;
pub type ClusterNodesCount = u16;
pub type StorageNodePubKey = AccountId32;
/// The type used to represent an MMR root hash.
pub type MmrRootHash = H256;
pub type BatchIndex = u16;

// ClusterParams includes Governance non-sensetive parameters only
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub struct ClusterParams<AccountId> {
	pub node_provider_auth_contract: Option<AccountId>,
	pub erasure_coding_required: u32,
	pub erasure_coding_total: u32,
	pub replication_total: u32,
}

#[cfg(feature = "std")]
impl<AccountId> Default for ClusterParams<AccountId> {
	fn default() -> Self {
		ClusterParams {
			node_provider_auth_contract: None,
			erasure_coding_required: 0,
			erasure_coding_total: 0,
			replication_total: 0,
		}
	}
}

// ClusterProtocolParams includes Governance sensitive parameters
#[derive(
	Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Default, Serialize, Deserialize,
)]
#[scale_info(skip_type_params(Balance, BlockNumber, T))]
pub struct ClusterProtocolParams<Balance, BlockNumber> {
	pub treasury_share: Perquintill,
	pub validators_share: Perquintill,
	pub cluster_reserve_share: Perquintill,
	pub storage_bond_size: Balance,
	pub storage_chill_delay: BlockNumber,
	pub storage_unbonding_delay: BlockNumber,
	pub unit_per_mb_stored: u128,
	pub unit_per_mb_streamed: u128,
	pub unit_per_put_request: u128,
	pub unit_per_get_request: u128,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub struct ClusterPricingParams {
	pub unit_per_mb_stored: u128,
	pub unit_per_mb_streamed: u128,
	pub unit_per_put_request: u128,
	pub unit_per_get_request: u128,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub struct ClusterFeesParams {
	pub treasury_share: Perquintill,
	pub validators_share: Perquintill,
	pub cluster_reserve_share: Perquintill,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub struct ClusterBondingParams<BlockNumber> {
	pub storage_bond_size: u128,
	pub storage_chill_delay: BlockNumber,
	pub storage_unbonding_delay: BlockNumber,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Serialize, Deserialize)]
pub enum NodePubKey {
	StoragePubKey(StorageNodePubKey),
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub enum NodeType {
	Storage = 1,
}

impl From<NodeType> for u8 {
	fn from(node_type: NodeType) -> Self {
		match node_type {
			NodeType::Storage => 1,
		}
	}
}

impl TryFrom<u8> for NodeType {
	type Error = ();
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			1 => Ok(NodeType::Storage),
			_ => Err(()),
		}
	}
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Serialize, Deserialize)]
pub enum StorageNodeMode {
	/// DDC Storage node operates with enabled caching in RAM and stores data in Hard Drive
	Full = 1,
	/// DDC Storage node operates with disabled caching in RAM and stores data in Hard Drive
	Storage = 2,
	/// DDC Storage node operates with enabled caching in RAM and doesn't store data in Hard Drive
	Cache = 3,
	// DAC node
	DAC = 4,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub struct StorageNodeParams {
	pub mode: StorageNodeMode,
	pub host: Vec<u8>,
	pub domain: Vec<u8>,
	pub ssl: bool,
	pub http_port: u16,
	pub grpc_port: u16,
	pub p2p_port: u16,
}

#[cfg(feature = "std")]
impl Default for StorageNodeParams {
	fn default() -> Self {
		StorageNodeParams {
			mode: StorageNodeMode::Full,
			host: Default::default(),
			domain: Default::default(),
			ssl: Default::default(),
			http_port: Default::default(),
			grpc_port: Default::default(),
			p2p_port: Default::default(),
		}
	}
}

// Params fields are always coming from extrinsic input
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub enum NodeParams {
	StorageParams(StorageNodeParams),
}

/// DDC cluster status
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Serialize, Deserialize)]
pub enum ClusterStatus {
	Unbonded,
	Bonded,
	Activated,
	Unbonding,
}

/// DDC node kind added to DDC cluster
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Serialize, Deserialize)]
pub enum ClusterNodeKind {
	Genesis,
	External,
}

/// DDC node status in to DDC cluster
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Serialize, Deserialize)]
pub enum ClusterNodeStatus {
	AwaitsValidation,
	ValidationSucceeded,
	ValidationFailed,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub struct ClusterNodeState<BlockNumber> {
	pub kind: ClusterNodeKind,
	pub status: ClusterNodeStatus,
	pub added_at: BlockNumber,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Default)]
pub struct ClusterNodesStats {
	pub await_validation: ClusterNodesCount,
	pub validation_succeeded: ClusterNodesCount,
	pub validation_failed: ClusterNodesCount,
}

/// Stores usage of customers
#[derive(PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, Default, Clone)]
pub struct CustomerUsage {
	pub transferred_bytes: u64,
	pub stored_bytes: u64,
	pub number_of_puts: u64,
	pub number_of_gets: u64,
}

/// Stores usage of node provider
#[derive(PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, Default, Clone)]
pub struct NodeUsage {
	pub transferred_bytes: u64,
	pub stored_bytes: u64,
	pub number_of_puts: u64,
	pub number_of_gets: u64,
}

#[derive(Debug, PartialEq)]
pub enum NodeRepositoryError {
	StorageNodeAlreadyExists,
	StorageNodeDoesNotExist,
}

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"cer!");
pub mod sr25519 {
	mod app_sr25519 {
		use sp_application_crypto::{app_crypto, sr25519};

		use crate::KEY_TYPE;
		app_crypto!(sr25519, KEY_TYPE);
	}

	sp_application_crypto::with_pair! {
		pub type AuthorityPair = app_sr25519::Pair;
	}
	pub type AuthoritySignature = app_sr25519::Signature;
	pub type AuthorityId = app_sr25519::Public;
}
pub mod crypto {
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};

	use super::KEY_TYPE;
	app_crypto!(sr25519, KEY_TYPE);

	pub struct OffchainIdentifierId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OffchainIdentifierId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for OffchainIdentifierId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

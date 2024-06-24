use frame_system::Config;
use polkadot_ckb_merkle_mountain_range::MerkleProof;
use sp_std::prelude::*;

use crate::{
	ActivityHash, BatchIndex, BucketId, ClusterId, CustomerUsage, DdcEra, MergeActivityHash,
	NodeUsage,
};

pub trait ValidatorVisitor<T: Config> {
	fn setup_validators(validators: Vec<T::AccountId>);
	fn is_ocw_validator(caller: T::AccountId) -> bool;
	fn is_customers_batch_valid(
		cluster_id: ClusterId,
		era: DdcEra,
		batch_index: BatchIndex,
		payers: &[(T::AccountId, BucketId, CustomerUsage)],
		proof: MerkleProof<ActivityHash, MergeActivityHash>,
		leaf_with_position: (u64, ActivityHash),
	) -> bool;
	fn is_providers_batch_valid(
		cluster_id: ClusterId,
		era: DdcEra,
		batch_index: BatchIndex,
		payees: &[(T::AccountId, BucketId, NodeUsage)],
		adjacent_hashes: &[ActivityHash],
	) -> bool;
}

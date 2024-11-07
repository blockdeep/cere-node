use frame_system::Config;
#[cfg(feature = "runtime-benchmarks")]
use scale_info::prelude::vec::Vec;

use crate::{
	BatchIndex, BucketId, ClusterId, CustomerUsage, DdcEra, MMRProof, NodePubKey, NodeUsage,
};

pub trait ValidatorVisitor<T: Config> {
	#[cfg(feature = "runtime-benchmarks")]
	fn setup_validators(validators: Vec<T::AccountId>);
	fn is_ocw_validator(caller: T::AccountId) -> bool;
	fn is_customers_batch_valid(
		cluster_id: ClusterId,
		era: DdcEra,
		batch_index: BatchIndex,
		max_batch_index: BatchIndex,
		payers: &[(NodePubKey, BucketId, CustomerUsage)],
		batch_proof: &MMRProof,
	) -> bool;
	fn is_providers_batch_valid(
		cluster_id: ClusterId,
		era: DdcEra,
		batch_index: BatchIndex,
		max_batch_index: BatchIndex,
		payees: &[(NodePubKey, NodeUsage)],
		batch_proof: &MMRProof,
	) -> bool;
}

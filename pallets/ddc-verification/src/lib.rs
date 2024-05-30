//! # DDC Verification Pallet
//!
//! The DDC Verification pallet is used to validate zk-SNARK Proof and Signature
//!
//! - [`Call`]
//! - [`Pallet`]

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use core::str;

use ddc_primitives::{
	traits::{ClusterManager, NodeVisitor, ValidatorVisitor},
	BatchIndex, ClusterId, CustomerUsage, DdcEra, MmrRootHash, NodeParams, NodePubKey, NodeUsage,
	StorageNodeMode, StorageNodeParams,
};
use frame_support::{
	pallet_prelude::*,
	traits::{Get, OneSessionHandler},
};
use frame_system::{
	offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer},
	pallet_prelude::*,
};
pub use pallet::*;
use scale_info::prelude::format;
use serde::{Deserialize, Serialize};
use sp_application_crypto::RuntimeAppPublic;
use sp_runtime::{offchain as rt_offchain, offchain::http, Percent};
use sp_std::prelude::*;

pub mod weights;
use crate::weights::WeightInfo;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use ddc_primitives::BucketId;
	use frame_support::PalletId;

	use super::*;

	/// The current storage version.
	const STORAGE_VERSION: frame_support::traits::StorageVersion =
		frame_support::traits::StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		#[pallet::constant]
		type MaxVerificationKeyLimit: Get<u32>;
		type WeightInfo: WeightInfo;
		type ClusterManager: ClusterManager<Self>;
		type NodeVisitor: NodeVisitor<Self>;
		type AuthorityId: Member
			+ Parameter
			+ RuntimeAppPublic
			+ Ord
			+ MaybeSerializeDeserialize
			+ Into<sp_core::sr25519::Public>
			+ From<sp_core::sr25519::Public>;

		type OffchainIdentifierId: AppCrypto<Self::Public, Self::Signature>;
		const MAJORITY: u8;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		BillingReportCreated { cluster_id: ClusterId, era: DdcEra },
		VerificationKeyStored { verification_key: Vec<u8> },
		PayoutBatchCreated { cluster_id: ClusterId, era: DdcEra },
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		BillingReportAlreadyExist,
		BadVerificationKey,
		BadRequest,
		NotAValidator,
		AlreadySigned,
		NodeRetrievalError,
		NodeUsageRetrievalError,
		ClusterToValidateRetrievalError,
		EraToValidateRetrievalError,
	}

	#[pallet::storage]
	#[pallet::getter(fn active_billing_reports)]
	pub type ActiveBillingReports<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, ClusterId, Blake2_128Concat, DdcEra, ReceiptParams>;

	#[pallet::storage]
	#[pallet::getter(fn payout_batch)]
	pub type PayoutBatch<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, ClusterId, Blake2_128Concat, DdcEra, PayoutData>;

	#[pallet::storage]
	#[pallet::getter(fn payout_validators)]
	pub type PayoutValidators<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		(ClusterId, DdcEra),
		Blake2_128Concat,
		MmrRootHash,
		Vec<T::AccountId>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn cluster_to_validate)]
	pub type ClusterToValidate<T: Config> = StorageValue<_, ClusterId>;

	#[pallet::storage]
	#[pallet::getter(fn era_to_validate)]
	pub type EraToValidate<T: Config> = StorageValue<_, DdcEra>;

	#[pallet::storage]
	#[pallet::getter(fn verification_key)]
	pub type VerificationKey<T: Config> =
		StorageValue<_, BoundedVec<u8, T::MaxVerificationKeyLimit>>;

	#[pallet::storage]
	#[pallet::getter(fn validator_set)]
	pub type ValidatorSet<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
	pub struct ReceiptParams {
		pub merkle_root_hash: MmrRootHash,
	}

	#[derive(Serialize, Deserialize, Debug, Clone)]
	pub(crate) struct NodeActivity {
		#[serde(rename = "totalBytesStored")]
		pub(crate) stored_bytes: u64,

		#[serde(rename = "totalBytesDelivered")]
		pub(crate) transferred_bytes: u64,

		#[serde(rename = "totalPutRequests")]
		pub(crate) number_of_puts: u64,

		#[serde(rename = "totalGetRequests")]
		pub(crate) number_of_gets: u64,

		#[serde(rename = "proof")]
		pub(crate) proof: Vec<u8>,
	}

	#[derive(Debug, Serialize, Deserialize, Clone)]
	pub struct CustomerActivity {
		#[serde(rename = "customerId")]
		pub customer_id: [u8; 32],

		#[serde(rename = "bucketId")]
		pub bucket_id: BucketId,

		#[serde(rename = "totalBytesStored")]
		pub stored_bytes: u64,

		#[serde(rename = "totalBytesDelivered")]
		pub transferred_bytes: u64,

		#[serde(rename = "totalPutRequests")]
		pub number_of_puts: u64,

		#[serde(rename = "totalGetRequests")]
		pub number_of_gets: u64,

		#[serde(rename = "proof")]
		pub proof: Vec<u8>,
	}

	impl From<CustomerActivity> for CustomerUsage {
		fn from(activity: CustomerActivity) -> Self {
			CustomerUsage {
				transferred_bytes: activity.transferred_bytes,
				stored_bytes: activity.stored_bytes,
				number_of_puts: activity.number_of_puts,
				number_of_gets: activity.number_of_gets,
			}
		}
	}

	impl From<NodeActivity> for NodeUsage {
		fn from(activity: NodeActivity) -> Self {
			NodeUsage {
				transferred_bytes: activity.transferred_bytes,
				stored_bytes: activity.stored_bytes,
				number_of_puts: activity.number_of_puts,
				number_of_gets: activity.number_of_gets,
			}
		}
	}

	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
	#[scale_info(skip_type_params(Hash))]
	pub struct PayoutData {
		pub hash: MmrRootHash,
	}

	macro_rules! unwrap_or_log_error {
		($result:expr, $error_msg:expr) => {
			match $result {
				Ok(val) => val,
				Err(err) => {
					log::error!("{}: {:?}", $error_msg, err);
					return;
				},
			}
		};
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_block_number: BlockNumberFor<T>) {
			log::info!("Hello from pallet-ocw.");

			let signer = Signer::<T, T::OffchainIdentifierId>::all_accounts();
			if !signer.can_sign() {
				log::error!("No local accounts available");
				return;
			}

			let results =
				signer.send_signed_transaction(|_account| Call::set_validate_payout_batch {
					cluster_id: Default::default(),
					era: DdcEra::default(),
					payout_data: PayoutData { hash: MmrRootHash::default() },
				});

			for (acc, res) in &results {
				match res {
					Ok(()) => log::info!("[{:?}] Submitted response", acc.id),
					Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
				}
			}

			let era_id = unwrap_or_log_error!(
				Self::get_era_to_validate(),
				"Error retrieving era to validate"
			);
			let cluster_id = unwrap_or_log_error!(
				Self::get_cluster_to_validate(),
				"Error retrieving cluster to validate"
			);
			let dac_nodes = unwrap_or_log_error!(
				Self::get_dac_nodes(&cluster_id),
				"Error retrieving dac nodes to validate"
			);
			let _nodes_usage = unwrap_or_log_error!(
				Self::fetch_nodes_usage_for_era(&cluster_id, era_id, &dac_nodes),
				"Error retrieving node activities to validate"
			);

			let _customers_usage = unwrap_or_log_error!(
				Self::fetch_customers_usage_for_era(&cluster_id, era_id, &dac_nodes),
				"Error retrieving customers activities to validate"
			);
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_era_to_validate() -> Result<DdcEra, Error<T>> {
			Self::era_to_validate().ok_or(Error::EraToValidateRetrievalError)
		}

		fn get_cluster_to_validate() -> Result<ClusterId, Error<T>> {
			Self::cluster_to_validate().ok_or(Error::ClusterToValidateRetrievalError)
		}

		pub(crate) fn fetch_customers_usage(
			_cluster_id: &ClusterId,
			era_id: DdcEra,
			node_params: &StorageNodeParams,
		) -> Result<Vec<CustomerActivity>, http::Error> {
			let scheme = if node_params.ssl { "https" } else { "http" };
			let host = str::from_utf8(&node_params.host).map_err(|_| http::Error::Unknown)?;
			let url = format!(
				"{}://{}:{}/activity/buckets?eraId={}",
				scheme, host, node_params.http_port, era_id
			);

			let request = http::Request::get(&url);
			let timeout =
				sp_io::offchain::timestamp().add(sp_runtime::offchain::Duration::from_millis(3000));
			let pending = request.deadline(timeout).send().map_err(|_| http::Error::IoError)?;

			let response =
				pending.try_wait(timeout).map_err(|_| http::Error::DeadlineReached)??;
			if response.code != 200 {
				return Err(http::Error::Unknown);
			}

			let body = response.body().collect::<Vec<u8>>();
			serde_json::from_slice(&body).map_err(|_| http::Error::Unknown)
		}

		pub(crate) fn fetch_node_usage(
			_cluster_id: &ClusterId,
			era_id: DdcEra,
			node_params: &StorageNodeParams,
		) -> Result<NodeActivity, http::Error> {
			let scheme = if node_params.ssl { "https" } else { "http" };
			let host = str::from_utf8(&node_params.host).map_err(|_| http::Error::Unknown)?;
			let url = format!(
				"{}://{}:{}/activity/node?eraId={}",
				scheme, host, node_params.http_port, era_id
			);

			let request = http::Request::get(&url);
			let timeout =
				sp_io::offchain::timestamp().add(rt_offchain::Duration::from_millis(3000));
			let pending = request.deadline(timeout).send().map_err(|_| http::Error::IoError)?;

			let response =
				pending.try_wait(timeout).map_err(|_| http::Error::DeadlineReached)??;
			if response.code != 200 {
				return Err(http::Error::Unknown);
			}

			let body = response.body().collect::<Vec<u8>>();
			serde_json::from_slice(&body).map_err(|_| http::Error::Unknown)
		}

		fn get_dac_nodes(
			cluster_id: &ClusterId,
		) -> Result<Vec<(NodePubKey, StorageNodeParams)>, Error<T>> {
			let mut dac_nodes = Vec::new();

			let nodes = T::ClusterManager::get_nodes(cluster_id)
				.map_err(|_| Error::<T>::NodeRetrievalError)?;

			// Iterate over each node
			for node_pub_key in nodes {
				// Get the node parameters
				if let Ok(NodeParams::StorageParams(storage_params)) =
					T::NodeVisitor::get_node_params(&node_pub_key)
				{
					// Check if the mode is StorageNodeMode::DAC
					if storage_params.mode == StorageNodeMode::DAC {
						// Add to the results if the mode matches
						dac_nodes.push((node_pub_key, storage_params));
					}
				}
			}

			Ok(dac_nodes)
		}

		fn fetch_nodes_usage_for_era(
			cluster_id: &ClusterId,
			era_id: DdcEra,
			dac_nodes: &[(NodePubKey, StorageNodeParams)],
		) -> Result<Vec<(NodePubKey, NodeActivity)>, Error<T>> {
			let mut node_usages = Vec::new();

			for (node_pub_key, node_params) in dac_nodes {
				let usage = Self::fetch_node_usage(cluster_id, era_id, node_params)
					.map_err(|_| Error::<T>::NodeUsageRetrievalError)?;

				node_usages.push((node_pub_key.clone(), usage));
			}

			Ok(node_usages)
		}

		fn fetch_customers_usage_for_era(
			cluster_id: &ClusterId,
			era_id: DdcEra,
			dac_nodes: &[(NodePubKey, StorageNodeParams)],
		) -> Result<Vec<(NodePubKey, Vec<CustomerActivity>)>, Error<T>> {
			let mut customers_usages = Vec::new();

			for (node_pub_key, node_params) in dac_nodes {
				let usage = Self::fetch_customers_usage(cluster_id, era_id, node_params)
					.map_err(|_| Error::<T>::NodeUsageRetrievalError)?;

				customers_usages.push((node_pub_key.clone(), usage));
			}

			Ok(customers_usages)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_billing_reports())]
		pub fn create_billing_reports(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			era: DdcEra,
			merkle_root_hash: MmrRootHash,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(
				ActiveBillingReports::<T>::get(cluster_id, era).is_none(),
				Error::<T>::BillingReportAlreadyExist
			);

			let receipt_params = ReceiptParams { merkle_root_hash };

			ActiveBillingReports::<T>::insert(cluster_id, era, receipt_params);

			Self::deposit_event(Event::<T>::BillingReportCreated { cluster_id, era });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::create_billing_reports())]
		pub fn set_verification_key(
			origin: OriginFor<T>,
			verification_key: Vec<u8>,
		) -> DispatchResult {
			ensure_root(origin)?;
			let bounded_verification_key: BoundedVec<u8, T::MaxVerificationKeyLimit> =
				verification_key
					.clone()
					.try_into()
					.map_err(|_| Error::<T>::BadVerificationKey)?;

			VerificationKey::<T>::put(bounded_verification_key);
			Self::deposit_event(Event::<T>::VerificationKeyStored { verification_key });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::create_billing_reports())]
		pub fn set_validate_payout_batch(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			era: DdcEra,
			payout_data: PayoutData,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let validators = <ValidatorSet<T>>::get();

			ensure!(validators.contains(&who.clone()), Error::<T>::NotAValidator);

			ensure!(
				!<PayoutValidators<T>>::get((cluster_id, era), payout_data.hash)
					.contains(&who.clone()),
				Error::<T>::AlreadySigned
			);

			<PayoutValidators<T>>::try_mutate(
				(cluster_id, era),
				payout_data.hash,
				|validators| -> DispatchResult {
					validators.push(who);
					Ok(())
				},
			)?;

			let percent = Percent::from_percent(T::MAJORITY);
			let threshold = percent * validators.len();

			let signed_validators = <PayoutValidators<T>>::get((cluster_id, era), payout_data.hash);

			if threshold < signed_validators.len() {
				PayoutBatch::<T>::insert(cluster_id, era, payout_data);
				Self::deposit_event(Event::<T>::PayoutBatchCreated { cluster_id, era });
			}

			Ok(())
		}
	}

	impl<T: Config> ValidatorVisitor<T> for Pallet<T> {
		fn setup_validators(validators: Vec<T::AccountId>) {
			ValidatorSet::<T>::put(validators);
		}
		fn get_active_validators() -> Vec<T::AccountId> {
			Self::validator_set()
		}

		fn is_customers_batch_valid(
			_cluster_id: ClusterId,
			_era: DdcEra,
			_batch_index: BatchIndex,
			_payers: Vec<(T::AccountId, CustomerUsage)>,
		) -> bool {
			true
		}
		fn is_providers_batch_valid(
			_cluster_id: ClusterId,
			_era: DdcEra,
			_batch_index: BatchIndex,
			_payees: Vec<(T::AccountId, NodeUsage)>,
		) -> bool {
			true
		}
	}

	impl<T: Config> sp_application_crypto::BoundToRuntimeAppPublic for Pallet<T> {
		type Public = T::AuthorityId;
	}

	impl<T: Config> OneSessionHandler<T::AccountId> for Pallet<T> {
		type Key = T::AuthorityId;

		fn on_genesis_session<'a, I: 'a>(validators: I)
		where
			I: Iterator<Item = (&'a T::AccountId, Self::Key)>,
		{
			let validators = validators
				.map(|(_, k)| T::AccountId::decode(&mut &k.into().encode()[..]).unwrap())
				.collect::<Vec<_>>();

			ValidatorSet::<T>::put(validators);
		}

		fn on_new_session<'a, I: 'a>(_changed: bool, validators: I, _queued_authorities: I)
		where
			I: Iterator<Item = (&'a T::AccountId, Self::Key)>,
		{
			let validators = validators
				.map(|(_, k)| T::AccountId::decode(&mut &k.into().encode()[..]).unwrap())
				.collect::<Vec<_>>();
			ValidatorSet::<T>::put(validators);
		}

		fn on_disabled(_i: u32) {}
	}
}

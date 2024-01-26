//! # DDC Nodes Pallet
//!
//! The DDC Clusters pallet is used to manage DDC Clusters
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## GenesisConfig
//!
//! The DDC Clusters pallet depends on the [`GenesisConfig`]. The
//! `GenesisConfig` is optional and allow to set some initial nodes in DDC.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]
#![feature(is_some_and)] // ToDo: delete at rustc > 1.70

pub mod weights;
use ddc_primitives::{
	traits::{
		cluster::{ClusterAdministrator, ClusterManager},
		node::NodeVisitor,
		pallet::GetDdcOrigin,
	},
	ClusterGovParams, ClusterId, ClusterNodeStatus, MIN_VALIDATED_NODES_COUNT,
};
use frame_support::{
	codec::{Decode, Encode},
	dispatch::{DispatchError, Dispatchable},
	pallet_prelude::*,
	traits::{
		schedule::DispatchTime, Currency, LockableCurrency, OriginTrait, StorePreimage,
		UnfilteredDispatchable,
	},
};
use frame_system::pallet_prelude::*;
pub use frame_system::Config as SysConfig;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{traits::AccountIdConversion, RuntimeDebug};
use sp_std::prelude::*;

pub type ProposalIndex = u32;
pub type MemberCount = u32;

use crate::weights::WeightInfo;

/// The balance type of this pallet.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Info for keeping track of a motion being voted on.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Votes<AccountId, BlockNumber> {
	/// The number of approval votes that are needed to pass the motion.
	threshold: MemberCount,
	/// The current set of voters that approved it.
	ayes: Vec<AccountId>,
	/// The current set of voters that rejected it.
	nays: Vec<AccountId>,
	/// The hard end time of this vote.
	end: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
	use ddc_primitives::NodePubKey;
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
	pub trait Config: frame_system::Config + pallet_referenda::Config {
		type PalletId: Get<PalletId>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		type WeightInfo: WeightInfo;
		type ClusterProposalDuration: Get<Self::BlockNumber>;
		type ClusterProposalCall: Parameter
			+ From<Call<Self>>
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ IsType<<Self as pallet_referenda::Config>::RuntimeCall>;

		type ClusterGovOrigin: GetDdcOrigin<Self>;
		type ClusterActivatorOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type ClusterAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type ClusterAdministrator: ClusterAdministrator<Self, BalanceOf<Self>>;
		type ClusterManager: ClusterManager<Self>;
		type NodeVisitor: NodeVisitor<Self>;
	}

	#[pallet::storage]
	#[pallet::getter(fn proposal_of)]
	pub type ClusterProposal<T: Config> =
		StorageMap<_, Identity, ClusterId, T::ClusterProposalCall, OptionQuery>;

	/// Votes on a given cluster proposal, if it is ongoing.
	#[pallet::storage]
	#[pallet::getter(fn voting)]
	pub type ClusterProposalVoting<T: Config> =
		StorageMap<_, Identity, ClusterId, Votes<T::AccountId, T::BlockNumber>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A motion (given hash) has been proposed (by given account) with a threshold (given
		/// `MemberCount`).
		Proposed { account: T::AccountId, cluster_id: ClusterId, threshold: MemberCount },
		/// A motion (given hash) has been voted on by given account, leaving
		/// a tally (yes votes and no votes given respectively as `MemberCount`).
		Voted {
			account: T::AccountId,
			proposal_hash: T::Hash,
			voted: bool,
			yes: MemberCount,
			no: MemberCount,
		},
		/// A motion was approved by the required threshold.
		Approved { proposal_hash: T::Hash },
		/// A motion was not approved by the required threshold.
		Disapproved { proposal_hash: T::Hash },
		/// A motion was executed; result will be `Ok` if it returned without error.
		Executed { proposal_hash: T::Hash, result: DispatchResult },
		/// A proposal was closed because its threshold was reached or after its duration was up.
		Closed { proposal_hash: T::Hash, yes: MemberCount, no: MemberCount },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account is not a member
		NotClusterMember,
		/// Account is not a cluster manager
		NotClusterManager,
		/// Account is not a member
		NotValidatedClusterMember,
		/// Cluster does not exist
		NoCluster,
		/// Proposal must exist
		ProposalMissing,
		/// Duplicate vote ignored
		DuplicateVote,
		/// The close call was made too early, before the end of the voting.
		TooEarly,
		AwaitsValidation,
		NotEnoughValidatedNodes,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn propose_activate_cluster(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			cluster_gov_params: ClusterGovParams<BalanceOf<T>, T::BlockNumber>,
		) -> DispatchResult {
			let caller_id = ensure_signed(origin)?;
			Self::ensure_cluster_manager(caller_id.clone(), cluster_id)?;

			let cluster_nodes_stats = T::ClusterManager::get_nodes_stats(&cluster_id)
				.map_err(|_| Error::<T>::NoCluster)?;
			ensure!(cluster_nodes_stats.await_validation == 0, Error::<T>::AwaitsValidation);
			ensure!(
				cluster_nodes_stats.validation_succeeded >= MIN_VALIDATED_NODES_COUNT,
				Error::<T>::NotEnoughValidatedNodes
			);

			let threshold = cluster_nodes_stats.validation_succeeded;

			let votes = {
				let end =
					frame_system::Pallet::<T>::block_number() + T::ClusterProposalDuration::get();
				Votes { threshold, ayes: vec![], nays: vec![], end }
			};
			let proposal: <T as Config>::ClusterProposalCall =
				T::ClusterProposalCall::from(Call::<T>::activate_cluster {
					cluster_id,
					cluster_gov_params,
				});

			<ClusterProposal<T>>::insert(cluster_id, proposal);
			<ClusterProposalVoting<T>>::insert(cluster_id, votes);
			Self::deposit_event(Event::Proposed { account: caller_id, cluster_id, threshold });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn vote_proposal(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			node_pub_key: NodePubKey,
		) -> DispatchResult {
			let caller_id = ensure_signed(origin)?;
			Self::ensure_validated_member(caller_id, cluster_id, node_pub_key)?;
			// todo: implement voting

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn close_proposal(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			node_pub_key: NodePubKey,
		) -> DispatchResult {
			let caller_id = ensure_signed(origin)?;
			Self::ensure_validated_member(caller_id, cluster_id, node_pub_key)?;
			// todo: check the local consensus on proposal
			Self::propose_public(cluster_id)?;

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn activate_cluster(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			cluster_gov_params: ClusterGovParams<BalanceOf<T>, T::BlockNumber>,
		) -> DispatchResult {
			T::ClusterActivatorOrigin::ensure_origin(origin)?;
			T::ClusterAdministrator::activate_cluster(cluster_id)?;
			T::ClusterAdministrator::update_cluster_gov_params(cluster_id, cluster_gov_params)
		}
	}

	impl<T: Config> Pallet<T> {
		fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		fn ensure_cluster_manager(
			origin: T::AccountId,
			cluster_id: ClusterId,
		) -> Result<(), DispatchError> {
			let cluster_manager = T::ClusterManager::get_manager_account_id(&cluster_id)
				.map_err(|_| Error::<T>::NoCluster)?;
			ensure!(origin == cluster_manager, Error::<T>::NotClusterManager);
			Ok(())
		}

		fn ensure_validated_member(
			origin: T::AccountId,
			cluster_id: ClusterId,
			node_pub_key: NodePubKey,
		) -> Result<(), DispatchError> {
			let cluster_manager = T::ClusterManager::get_manager_account_id(&cluster_id)
				.map_err(|_| Error::<T>::NoCluster)?;

			if origin == cluster_manager {
				return Ok(())
			}

			let is_validated = T::ClusterManager::contains_node(
				&cluster_id,
				&node_pub_key,
				Some(ClusterNodeStatus::ValidationSucceeded),
			);

			if !is_validated {
				return Err(Error::<T>::NotValidatedClusterMember.into())
			}

			let node_provider = T::NodeVisitor::get_node_provider_id(&node_pub_key)?;
			if origin == node_provider {
				return Ok(())
			}

			Err(Error::<T>::NotValidatedClusterMember.into())
		}

		fn propose_public(cluster_id: ClusterId) -> DispatchResult {
			let proposal = <ClusterProposal<T>>::try_get(cluster_id)
				.map_err(|_| Error::<T>::ProposalMissing)?;

			let call: <T as pallet_referenda::Config>::RuntimeCall = proposal.into();
			let bounded_call =
				T::Preimages::bound(call).map_err(|_| Error::<T>::ProposalMissing)?;

			let cluster_gov_origin = T::ClusterGovOrigin::get();
			let pallets_origin: <T::RuntimeOrigin as OriginTrait>::PalletsOrigin =
				cluster_gov_origin.caller().clone();
			let referenda_call = pallet_referenda::Call::<T>::submit {
				proposal_origin: Box::new(pallets_origin),
				proposal: bounded_call,
				enactment_moment: DispatchTime::After(T::BlockNumber::from(1u32)),
			};

			referenda_call
				.dispatch_bypass_filter(frame_system::RawOrigin::Signed(Self::account_id()).into())
				.map(|_| ())
				.map_err(|e| e.error)?;

			Ok(())
		}
	}
}
//! # DDC Payouts Pallet
//!
//! The DDC Payouts pallet is used to distribute payouts based on DAC validation
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## GenesisConfig
//!
//! The DDC Payouts pallet depends on the [`GenesisConfig`]. The
//! `GenesisConfig` is optional and allow to set some initial nodes in DDC.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

pub mod weights;

use crate::weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

use ddc_primitives::{
	traits::{
		cluster::{ClusterCreator as ClusterCreatorType, ClusterProtocol as ClusterProtocolType},
		customer::{
			CustomerCharger as CustomerChargerType, CustomerDepositor as CustomerDepositorType,
		},
		pallet::PalletVisitor as PalletVisitorType,
		payout::PayoutVisitor,
	},
	BatchIndex, BucketId, ClusterId, CustomerUsage, DdcEra, MMRProof, NodeUsage, PayoutError,
	PayoutState, MAX_PAYOUT_BATCH_COUNT, MAX_PAYOUT_BATCH_SIZE, MILLICENTS,
};
use frame_election_provider_support::SortedListProvider;
use frame_support::{
	pallet_prelude::*,
	parameter_types,
	sp_runtime::SaturatedConversion,
	traits::{Currency, ExistenceRequirement, LockableCurrency},
	BoundedBTreeSet,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_runtime::{traits::Convert, PerThing, Perquintill};
use sp_std::prelude::*;

/// Stores reward in tokens(units) of node provider as per NodeUsage
#[derive(PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, Default, Clone)]
pub struct NodeReward {
	pub transfer: u128, // reward in tokens for NodeUsage::transferred_bytes
	pub storage: u128,  // reward in tokens for NodeUsage::stored_bytes
	pub puts: u128,     // reward in tokens for NodeUsage::number_of_puts
	pub gets: u128,     // reward in tokens for NodeUsage::number_of_gets
}

#[derive(PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, Default, Clone)]
pub struct BillingReportDebt {
	pub cluster_id: ClusterId,
	pub era: DdcEra,
	pub batch_index: BatchIndex,
	pub amount: u128,
}

/// Stores charge in tokens(units) of customer as per CustomerUsage
#[derive(PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, Default, Clone)]
pub struct CustomerCharge {
	pub transfer: u128, // charge in tokens for CustomerUsage::transferred_bytes
	pub storage: u128,  // charge in tokens for CustomerUsage::stored_bytes
	pub puts: u128,     // charge in tokens for CustomerUsage::number_of_puts
	pub gets: u128,     // charge in tokens for CustomerUsage::number_of_gets
}

/// The balance type of this pallet.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type VoteScoreOf<T> =
	<<T as pallet::Config>::NominatorsAndValidatorsList as frame_election_provider_support::SortedListProvider<
		<T as frame_system::Config>::AccountId,
	>>::Score;

parameter_types! {
	pub MaxBatchesCount: u16 = MAX_PAYOUT_BATCH_COUNT;
	pub MaxDust: u128 = MILLICENTS;
	pub MaxBatchSize: u16 = MAX_PAYOUT_BATCH_SIZE;
}

#[frame_support::pallet]
pub mod pallet {
	use ddc_primitives::traits::ValidatorVisitor;
	use frame_support::PalletId;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AccountIdConversion, Zero};

	use super::*;

	/// The current storage version.
	const STORAGE_VERSION: frame_support::traits::StorageVersion =
		frame_support::traits::StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		type Currency: LockableCurrency<Self::AccountId, Moment = BlockNumberFor<Self>>;
		type CustomerCharger: CustomerChargerType<Self>;
		type CustomerDepositor: CustomerDepositorType<Self>;
		type TreasuryVisitor: PalletVisitorType<Self>;
		type ClusterProtocol: ClusterProtocolType<Self, BalanceOf<Self>>;
		type NominatorsAndValidatorsList: SortedListProvider<Self::AccountId>;
		type ClusterCreator: ClusterCreatorType<Self, BalanceOf<Self>>;
		type WeightInfo: WeightInfo;
		type VoteScoreToU64: Convert<VoteScoreOf<Self>, u64>;
		type ValidatorVisitor: ValidatorVisitor<Self>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		BillingReportInitialized {
			cluster_id: ClusterId,
			era: DdcEra,
		},
		ChargingStarted {
			cluster_id: ClusterId,
			era: DdcEra,
		},
		Charged {
			cluster_id: ClusterId,
			era: DdcEra,
			batch_index: BatchIndex,
			customer_id: T::AccountId,
			bucket_id: BucketId,
			amount: u128,
		},
		ChargeFailed {
			cluster_id: ClusterId,
			era: DdcEra,
			batch_index: BatchIndex,
			customer_id: T::AccountId,
			bucket_id: BucketId,
			charged: u128,
			expected_to_charge: u128,
		},
		Indebted {
			cluster_id: ClusterId,
			era: DdcEra,
			batch_index: BatchIndex,
			customer_id: T::AccountId,
			bucket_id: BucketId,
			amount: u128,
		},
		ChargingFinished {
			cluster_id: ClusterId,
			era: DdcEra,
		},
		TreasuryFeesCollected {
			cluster_id: ClusterId,
			era: DdcEra,
			amount: u128,
		},
		ClusterReserveFeesCollected {
			cluster_id: ClusterId,
			era: DdcEra,
			amount: u128,
		},
		ValidatorFeesCollected {
			cluster_id: ClusterId,
			era: DdcEra,
			amount: u128,
		},
		RewardingStarted {
			cluster_id: ClusterId,
			era: DdcEra,
		},
		Rewarded {
			cluster_id: ClusterId,
			era: DdcEra,
			batch_index: BatchIndex,
			node_provider_id: T::AccountId,
			bucket_id: BucketId,
			rewarded: u128,
			expected_to_reward: u128,
		},
		NotDistributedReward {
			cluster_id: ClusterId,
			era: DdcEra,
			batch_index: BatchIndex,
			node_provider_id: T::AccountId,
			bucket_id: BucketId,
			expected_reward: u128,
			distributed_reward: BalanceOf<T>,
		},
		NotDistributedOverallReward {
			cluster_id: ClusterId,
			era: DdcEra,
			expected_reward: u128,
			total_distributed_reward: u128,
		},
		RewardingFinished {
			cluster_id: ClusterId,
			era: DdcEra,
		},
		BillingReportFinalized {
			cluster_id: ClusterId,
			era: DdcEra,
		},
		ChargeError {
			cluster_id: ClusterId,
			era: DdcEra,
			batch_index: BatchIndex,
			customer_id: T::AccountId,
			amount: u128,
			error: DispatchError,
		},
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		BillingReportDoesNotExist,
		NotExpectedState,
		Unauthorised,
		BatchIndexAlreadyProcessed,
		BatchIndexIsOutOfRange,
		BatchesMissed,
		BatchIndexOverflow,
		BoundedVecOverflow,
		ArithmeticOverflow,
		NotExpectedClusterState,
		BatchSizeIsOutOfBounds,
		ScoreRetrievalError,
		BadRequest,
		BatchValidationFailed,
	}

	#[pallet::storage]
	#[pallet::getter(fn active_billing_reports)]
	pub type ActiveBillingReports<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ClusterId,
		Blake2_128Concat,
		DdcEra,
		BillingReport<T>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn debtor_customers)]
	pub type DebtorCustomers<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, ClusterId, Blake2_128Concat, T::AccountId, u128>;

	#[pallet::storage]
	#[pallet::getter(fn owing_providers)]
	pub type OwingProviders<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, ClusterId, Blake2_128Concat, T::AccountId, u128>;

	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
	#[scale_info(skip_type_params(T))]
	pub struct BillingReport<T: Config> {
		pub state: PayoutState,
		pub vault: T::AccountId,
		pub start_era: i64,
		pub end_era: i64,
		pub total_customer_charge: CustomerCharge,
		pub total_distributed_reward: u128,
		pub total_node_usage: NodeUsage,
		// stage 1
		pub charging_max_batch_index: BatchIndex,
		pub charging_processed_batches: BoundedBTreeSet<BatchIndex, MaxBatchesCount>,
		// stage 2
		pub rewarding_max_batch_index: BatchIndex,
		pub rewarding_processed_batches: BoundedBTreeSet<BatchIndex, MaxBatchesCount>,
	}

	impl<T: pallet::Config> Default for BillingReport<T> {
		fn default() -> Self {
			Self {
				state: PayoutState::default(),
				vault: T::PalletId::get().into_account_truncating(),
				start_era: Zero::zero(),
				end_era: Zero::zero(),
				total_customer_charge: CustomerCharge::default(),
				total_distributed_reward: Zero::zero(),
				total_node_usage: NodeUsage::default(),
				charging_max_batch_index: Zero::zero(),
				charging_processed_batches: BoundedBTreeSet::default(),
				rewarding_max_batch_index: Zero::zero(),
				rewarding_processed_batches: BoundedBTreeSet::default(),
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// todo! remove extrensics from payout pallet and factor the extrensics implementation into
		// PayoutProcessor trait
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::begin_billing_report())]
		pub fn begin_billing_report(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			era: DdcEra,
			start_era: i64,
			end_era: i64,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(T::ValidatorVisitor::is_ocw_validator(caller), Error::<T>::Unauthorised); //
																				  // todo! need to refactor this

			ensure!(
				ActiveBillingReports::<T>::try_get(cluster_id, era).is_err(),
				Error::<T>::NotExpectedState
			);

			ensure!(end_era > start_era, Error::<T>::BadRequest);

			let billing_report = BillingReport::<T> {
				vault: Self::account_id(),
				state: PayoutState::Initialized,
				start_era,
				end_era,
				..Default::default()
			};
			ActiveBillingReports::<T>::insert(cluster_id, era, billing_report);

			Self::deposit_event(Event::<T>::BillingReportInitialized { cluster_id, era });

			Ok(())
		}

		// todo! remove extrensics from payout pallet and factor the extrensics implementation into
		// PayoutProcessor trait
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::begin_charging_customers())]
		pub fn begin_charging_customers(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			era: DdcEra,
			max_batch_index: BatchIndex,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(T::ValidatorVisitor::is_ocw_validator(caller), Error::<T>::Unauthorised); //
																				  // todo! need to refactor this

			ensure!(max_batch_index < MaxBatchesCount::get(), Error::<T>::BatchIndexOverflow);

			let mut billing_report = ActiveBillingReports::<T>::try_get(cluster_id, era)
				.map_err(|_| Error::<T>::BillingReportDoesNotExist)?;

			ensure!(billing_report.state == PayoutState::Initialized, Error::<T>::NotExpectedState);

			billing_report.charging_max_batch_index = max_batch_index;
			billing_report.state = PayoutState::ChargingCustomers;
			ActiveBillingReports::<T>::insert(cluster_id, era, billing_report);

			Self::deposit_event(Event::<T>::ChargingStarted { cluster_id, era });

			Ok(())
		}

		// todo! remove extrensics from payout pallet and factor the extrensics implementation into
		// + pass values by reference PayoutProcessor trait
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::send_charging_customers_batch(payers.len().saturated_into()))]
		pub fn send_charging_customers_batch(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			era: DdcEra,
			batch_index: BatchIndex,
			payers: Vec<(T::AccountId, BucketId, CustomerUsage)>,
			batch_proof: MMRProof,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(T::ValidatorVisitor::is_ocw_validator(caller), Error::<T>::Unauthorised);

			ensure!(
				!payers.is_empty() && payers.len() <= MaxBatchSize::get() as usize,
				Error::<T>::BatchSizeIsOutOfBounds
			);

			let billing_report = ActiveBillingReports::<T>::try_get(cluster_id, era)
				.map_err(|_| Error::<T>::BillingReportDoesNotExist)?;

			ensure!(
				billing_report.state == PayoutState::ChargingCustomers,
				Error::<T>::NotExpectedState
			);
			ensure!(
				billing_report.charging_max_batch_index >= batch_index,
				Error::<T>::BatchIndexIsOutOfRange
			);
			ensure!(
				!billing_report.charging_processed_batches.contains(&batch_index),
				Error::<T>::BatchIndexAlreadyProcessed
			);

			ensure!(
				T::ValidatorVisitor::is_customers_batch_valid(
					cluster_id,
					era,
					batch_index,
					&payers,
					&batch_proof
				),
				Error::<T>::BatchValidationFailed
			);

			let mut updated_billing_report = billing_report;
			for (customer_id, bucket_id, customer_usage) in payers {
				let mut customer_charge = get_customer_charge::<T>(
					cluster_id,
					&customer_usage,
					updated_billing_report.start_era,
					updated_billing_report.end_era,
				)?;
				let total_customer_charge = (|| -> Option<u128> {
					customer_charge
						.transfer
						.checked_add(customer_charge.storage)?
						.checked_add(customer_charge.puts)?
						.checked_add(customer_charge.gets)
				})()
				.ok_or(Error::<T>::ArithmeticOverflow)?;

				let amount_actually_charged = match T::CustomerCharger::charge_content_owner(
					&cluster_id,
					bucket_id,
					customer_id.clone(),
					updated_billing_report.vault.clone(),
					&customer_usage,
					total_customer_charge,
				) {
					Ok(actually_charged) => actually_charged,
					Err(e) => {
						Self::deposit_event(Event::<T>::ChargeError {
							cluster_id,
							era,
							batch_index,
							customer_id: customer_id.clone(),
							amount: total_customer_charge,
							error: e,
						});
						0
					},
				};

				if amount_actually_charged < total_customer_charge {
					// debt
					let mut customer_debt =
						DebtorCustomers::<T>::try_get(cluster_id, customer_id.clone())
							.unwrap_or_else(|_| Zero::zero());

					let debt = total_customer_charge
						.checked_sub(amount_actually_charged)
						.ok_or(Error::<T>::ArithmeticOverflow)?;

					customer_debt =
						customer_debt.checked_add(debt).ok_or(Error::<T>::ArithmeticOverflow)?;

					DebtorCustomers::<T>::insert(cluster_id, customer_id.clone(), customer_debt);

					Self::deposit_event(Event::<T>::Indebted {
						cluster_id,
						era,
						batch_index,
						customer_id: customer_id.clone(),
						bucket_id,
						amount: debt,
					});

					Self::deposit_event(Event::<T>::ChargeFailed {
						cluster_id,
						era,
						batch_index,
						customer_id,
						bucket_id,
						charged: amount_actually_charged,
						expected_to_charge: total_customer_charge,
					});

					// something was charged and should be added
					// calculate ratio
					let ratio =
						Perquintill::from_rational(amount_actually_charged, total_customer_charge);

					customer_charge.storage = ratio * customer_charge.storage;
					customer_charge.transfer = ratio * customer_charge.transfer;
					customer_charge.gets = ratio * customer_charge.gets;
					customer_charge.puts = ratio * customer_charge.puts;
				} else {
					Self::deposit_event(Event::<T>::Charged {
						cluster_id,
						era,
						batch_index,
						customer_id,
						bucket_id,
						amount: total_customer_charge,
					});
				}

				updated_billing_report.total_customer_charge.storage = updated_billing_report
					.total_customer_charge
					.storage
					.checked_add(customer_charge.storage)
					.ok_or(Error::<T>::ArithmeticOverflow)?;

				updated_billing_report.total_customer_charge.transfer = updated_billing_report
					.total_customer_charge
					.transfer
					.checked_add(customer_charge.transfer)
					.ok_or(Error::<T>::ArithmeticOverflow)?;

				updated_billing_report.total_customer_charge.puts = updated_billing_report
					.total_customer_charge
					.puts
					.checked_add(customer_charge.puts)
					.ok_or(Error::<T>::ArithmeticOverflow)?;

				updated_billing_report.total_customer_charge.gets = updated_billing_report
					.total_customer_charge
					.gets
					.checked_add(customer_charge.gets)
					.ok_or(Error::<T>::ArithmeticOverflow)?;
			}

			updated_billing_report
				.charging_processed_batches
				.try_insert(batch_index)
				.map_err(|_| Error::<T>::BoundedVecOverflow)?;

			ActiveBillingReports::<T>::insert(cluster_id, era, updated_billing_report);

			Ok(())
		}

		// todo! remove extrensics from payout pallet and factor the extrensics implementation into
		// PayoutProcessor trait
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::end_charging_customers())]
		pub fn end_charging_customers(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			era: DdcEra,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(T::ValidatorVisitor::is_ocw_validator(caller), Error::<T>::Unauthorised);

			let mut billing_report = ActiveBillingReports::<T>::try_get(cluster_id, era)
				.map_err(|_| Error::<T>::BillingReportDoesNotExist)?;

			ensure!(
				billing_report.state == PayoutState::ChargingCustomers,
				Error::<T>::NotExpectedState
			);
			Self::validate_batches(
				&billing_report.charging_processed_batches,
				&billing_report.charging_max_batch_index,
			)?;

			Self::deposit_event(Event::<T>::ChargingFinished { cluster_id, era });

			// deduct fees
			let fees = T::ClusterProtocol::get_fees_params(&cluster_id)
				.map_err(|_| Error::<T>::NotExpectedClusterState)?;

			let total_customer_charge = (|| -> Option<u128> {
				billing_report
					.total_customer_charge
					.transfer
					.checked_add(billing_report.total_customer_charge.storage)?
					.checked_add(billing_report.total_customer_charge.puts)?
					.checked_add(billing_report.total_customer_charge.gets)
			})()
			.ok_or(Error::<T>::ArithmeticOverflow)?;

			let treasury_fee = fees.treasury_share * total_customer_charge;
			let validators_fee = fees.validators_share * total_customer_charge;
			let cluster_reserve_fee = fees.cluster_reserve_share * total_customer_charge;

			if treasury_fee > 0 {
				charge_treasury_fees::<T>(
					treasury_fee,
					&billing_report.vault,
					&T::TreasuryVisitor::get_account_id(),
				)?;

				Self::deposit_event(Event::<T>::TreasuryFeesCollected {
					cluster_id,
					era,
					amount: treasury_fee,
				});
			}

			if cluster_reserve_fee > 0 {
				charge_cluster_reserve_fees::<T>(
					cluster_reserve_fee,
					&billing_report.vault,
					&T::ClusterProtocol::get_reserve_account_id(&cluster_id)
						.map_err(|_| Error::<T>::NotExpectedClusterState)?,
				)?;
				Self::deposit_event(Event::<T>::ClusterReserveFeesCollected {
					cluster_id,
					era,
					amount: cluster_reserve_fee,
				});
			}

			if validators_fee > 0 {
				charge_validator_fees::<T>(validators_fee, &billing_report.vault)?;
				Self::deposit_event(Event::<T>::ValidatorFeesCollected {
					cluster_id,
					era,
					amount: validators_fee,
				});
			}

			// 1 - (X + Y + Z) > 0, 0 < X + Y + Z < 1
			let total_left_from_one =
				(fees.treasury_share + fees.validators_share + fees.cluster_reserve_share)
					.left_from_one();

			if !total_left_from_one.is_zero() {
				// X * Z < X, 0 < Z < 1
				billing_report.total_customer_charge.transfer =
					total_left_from_one * billing_report.total_customer_charge.transfer;
				billing_report.total_customer_charge.storage =
					total_left_from_one * billing_report.total_customer_charge.storage;
				billing_report.total_customer_charge.puts =
					total_left_from_one * billing_report.total_customer_charge.puts;
				billing_report.total_customer_charge.gets =
					total_left_from_one * billing_report.total_customer_charge.gets;
			}

			billing_report.state = PayoutState::CustomersChargedWithFees;
			ActiveBillingReports::<T>::insert(cluster_id, era, billing_report);

			Ok(())
		}

		// todo! remove extrensics from payout pallet and factor the extrensics implementation into
		// PayoutProcessor trait
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::begin_rewarding_providers())]
		pub fn begin_rewarding_providers(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			era: DdcEra,
			max_batch_index: BatchIndex,
			total_node_usage: NodeUsage,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(T::ValidatorVisitor::is_ocw_validator(caller), Error::<T>::Unauthorised);

			ensure!(max_batch_index < MaxBatchesCount::get(), Error::<T>::BatchIndexOverflow);

			let mut billing_report = ActiveBillingReports::<T>::try_get(cluster_id, era)
				.map_err(|_| Error::<T>::BillingReportDoesNotExist)?;

			ensure!(
				billing_report.state == PayoutState::CustomersChargedWithFees,
				Error::<T>::NotExpectedState
			);

			billing_report.total_node_usage = total_node_usage;
			billing_report.rewarding_max_batch_index = max_batch_index;
			billing_report.state = PayoutState::RewardingProviders;
			ActiveBillingReports::<T>::insert(cluster_id, era, billing_report);

			Self::deposit_event(Event::<T>::RewardingStarted { cluster_id, era });

			Ok(())
		}

		// todo! remove extrensics from payout pallet and factor the extrensics implementation into
		// + pass values by reference PayoutProcessor trait
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::send_rewarding_providers_batch(payees.len().saturated_into()))]
		pub fn send_rewarding_providers_batch(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			era: DdcEra,
			batch_index: BatchIndex,
			payees: Vec<(T::AccountId, BucketId, NodeUsage)>,
			batch_proof: MMRProof,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(T::ValidatorVisitor::is_ocw_validator(caller), Error::<T>::Unauthorised);

			ensure!(
				!payees.is_empty() && payees.len() <= MaxBatchSize::get() as usize,
				Error::<T>::BatchSizeIsOutOfBounds
			);

			let billing_report = ActiveBillingReports::<T>::try_get(cluster_id, era)
				.map_err(|_| Error::<T>::BillingReportDoesNotExist)?;

			ensure!(
				billing_report.state == PayoutState::RewardingProviders,
				Error::<T>::NotExpectedState
			);
			ensure!(
				billing_report.rewarding_max_batch_index >= batch_index,
				Error::<T>::BatchIndexIsOutOfRange
			);
			ensure!(
				!billing_report.rewarding_processed_batches.contains(&batch_index),
				Error::<T>::BatchIndexAlreadyProcessed
			);

			ensure!(
				T::ValidatorVisitor::is_providers_batch_valid(
					cluster_id,
					era,
					batch_index,
					&payees,
					&batch_proof
				),
				Error::<T>::BatchValidationFailed
			);

			let max_dust = MaxDust::get().saturated_into::<BalanceOf<T>>();
			let mut updated_billing_report = billing_report.clone();
			for (node_provider_id, bucket_id, node_usage) in payees {
				let node_reward = get_node_reward(
					&node_usage,
					&billing_report.total_node_usage,
					&billing_report.total_customer_charge,
				)
				.ok_or(Error::<T>::ArithmeticOverflow)?;
				let amount_to_reward = (|| -> Option<u128> {
					node_reward
						.transfer
						.checked_add(node_reward.storage)?
						.checked_add(node_reward.puts)?
						.checked_add(node_reward.gets)
				})()
				.ok_or(Error::<T>::ArithmeticOverflow)?;

				let mut reward_ = amount_to_reward;
				let mut reward: BalanceOf<T> = amount_to_reward.saturated_into::<BalanceOf<T>>();
				if amount_to_reward > 0 {
					let vault_balance = <T as pallet::Config>::Currency::free_balance(
						&updated_billing_report.vault,
					) - <T as pallet::Config>::Currency::minimum_balance();

					// 10000000000001 > 10000000000000 but is still ok
					if reward > vault_balance {
						if reward - vault_balance > max_dust {
							Self::deposit_event(Event::<T>::NotDistributedReward {
								cluster_id,
								era,
								batch_index,
								node_provider_id: node_provider_id.clone(),
								bucket_id,
								expected_reward: amount_to_reward,
								distributed_reward: vault_balance,
							});
						}

						reward = vault_balance;
					}

					<T as pallet::Config>::Currency::transfer(
						&updated_billing_report.vault,
						&node_provider_id,
						reward,
						ExistenceRequirement::AllowDeath,
					)?;

					reward_ = reward.saturated_into::<u128>();

					updated_billing_report.total_distributed_reward = updated_billing_report
						.total_distributed_reward
						.checked_add(reward_)
						.ok_or(Error::<T>::ArithmeticOverflow)?;
				}

				T::CustomerCharger::inc_total_node_usage(&cluster_id, bucket_id, &node_usage)?;

				Self::deposit_event(Event::<T>::Rewarded {
					cluster_id,
					era,
					batch_index,
					node_provider_id,
					bucket_id,
					rewarded: reward_,
					expected_to_reward: amount_to_reward,
				});
			}

			updated_billing_report
				.rewarding_processed_batches
				.try_insert(batch_index)
				.map_err(|_| Error::<T>::BoundedVecOverflow)?;

			ActiveBillingReports::<T>::insert(cluster_id, era, updated_billing_report);

			Ok(())
		}

		// todo! remove extrensics from payout pallet and factor the extrensics implementation into
		// PayoutProcessor trait
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::end_rewarding_providers())]
		pub fn end_rewarding_providers(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			era: DdcEra,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(T::ValidatorVisitor::is_ocw_validator(caller), Error::<T>::Unauthorised);

			let mut billing_report = ActiveBillingReports::<T>::try_get(cluster_id, era)
				.map_err(|_| Error::<T>::BillingReportDoesNotExist)?;

			ensure!(
				billing_report.state == PayoutState::RewardingProviders,
				Error::<T>::NotExpectedState
			);

			Self::validate_batches(
				&billing_report.rewarding_processed_batches,
				&billing_report.rewarding_max_batch_index,
			)?;

			let expected_amount_to_reward = (|| -> Option<u128> {
				billing_report
					.total_customer_charge
					.transfer
					.checked_add(billing_report.total_customer_charge.storage)?
					.checked_add(billing_report.total_customer_charge.puts)?
					.checked_add(billing_report.total_customer_charge.gets)
			})()
			.ok_or(Error::<T>::ArithmeticOverflow)?;

			if expected_amount_to_reward - billing_report.total_distributed_reward > MaxDust::get()
			{
				Self::deposit_event(Event::<T>::NotDistributedOverallReward {
					cluster_id,
					era,
					expected_reward: expected_amount_to_reward,
					total_distributed_reward: billing_report.total_distributed_reward,
				});
			}

			billing_report.state = PayoutState::ProvidersRewarded;
			ActiveBillingReports::<T>::insert(cluster_id, era, billing_report);

			Self::deposit_event(Event::<T>::RewardingFinished { cluster_id, era });

			Ok(())
		}

		// todo! remove extrensics from payout pallet and factor the extrensics implementation into
		// PayoutProcessor trait
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::end_billing_report())]
		pub fn end_billing_report(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			era: DdcEra,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(T::ValidatorVisitor::is_ocw_validator(caller), Error::<T>::Unauthorised);

			let mut billing_report = ActiveBillingReports::<T>::try_get(cluster_id, era)
				.map_err(|_| Error::<T>::BillingReportDoesNotExist)?;

			ensure!(
				billing_report.state == PayoutState::ProvidersRewarded,
				Error::<T>::NotExpectedState
			);

			billing_report.charging_processed_batches.clear();
			billing_report.rewarding_processed_batches.clear();
			billing_report.state = PayoutState::Finalized;

			ActiveBillingReports::<T>::insert(cluster_id, era, billing_report);
			Self::deposit_event(Event::<T>::BillingReportFinalized { cluster_id, era });

			Ok(())
		}
	}

	fn charge_treasury_fees<T: Config>(
		treasury_fee: u128,
		vault: &T::AccountId,
		treasury_vault: &T::AccountId,
	) -> DispatchResult {
		let amount_to_deduct = treasury_fee.saturated_into::<BalanceOf<T>>();
		<T as pallet::Config>::Currency::transfer(
			vault,
			treasury_vault,
			amount_to_deduct,
			ExistenceRequirement::AllowDeath,
		)
	}

	fn charge_cluster_reserve_fees<T: Config>(
		cluster_reserve_fee: u128,
		vault: &T::AccountId,
		reserve_vault: &T::AccountId,
	) -> DispatchResult {
		let amount_to_deduct = cluster_reserve_fee.saturated_into::<BalanceOf<T>>();
		<T as pallet::Config>::Currency::transfer(
			vault,
			reserve_vault,
			amount_to_deduct,
			ExistenceRequirement::AllowDeath,
		)
	}

	fn get_current_exposure_ratios<T: Config>(
	) -> Result<Vec<(T::AccountId, Perquintill)>, DispatchError> {
		let mut total_score = 0;
		let mut individual_scores: Vec<(T::AccountId, u64)> = Vec::new();
		for staker_id in T::NominatorsAndValidatorsList::iter() {
			let s = T::NominatorsAndValidatorsList::get_score(&staker_id)
				.map_err(|_| Error::<T>::ScoreRetrievalError)?;
			let score = T::VoteScoreToU64::convert(s);
			total_score += score;

			individual_scores.push((staker_id, score));
		}

		let mut result = Vec::new();
		for (staker_id, score) in individual_scores {
			let ratio = Perquintill::from_rational(score, total_score);
			result.push((staker_id, ratio));
		}

		Ok(result)
	}

	fn charge_validator_fees<T: Config>(
		validators_fee: u128,
		vault: &T::AccountId,
	) -> DispatchResult {
		let stakers = get_current_exposure_ratios::<T>()?;

		for (staker_id, ratio) in stakers.iter() {
			let amount_to_deduct = *ratio * validators_fee;

			<T as pallet::Config>::Currency::transfer(
				vault,
				staker_id,
				amount_to_deduct.saturated_into::<BalanceOf<T>>(),
				ExistenceRequirement::AllowDeath,
			)?;
		}

		Ok(())
	}

	fn get_node_reward(
		node_usage: &NodeUsage,
		total_nodes_usage: &NodeUsage,
		total_customer_charge: &CustomerCharge,
	) -> Option<NodeReward> {
		let mut node_reward = NodeReward::default();

		let mut ratio = Perquintill::from_rational(
			node_usage.transferred_bytes as u128,
			total_nodes_usage.transferred_bytes as u128,
		);

		// ratio multiplied by X will be > 0, < X no overflow
		node_reward.transfer = ratio * total_customer_charge.transfer;

		ratio = Perquintill::from_rational(
			node_usage.stored_bytes as u128,
			total_nodes_usage.stored_bytes as u128,
		);
		node_reward.storage = ratio * total_customer_charge.storage;

		ratio =
			Perquintill::from_rational(node_usage.number_of_puts, total_nodes_usage.number_of_puts);
		node_reward.puts = ratio * total_customer_charge.puts;

		ratio =
			Perquintill::from_rational(node_usage.number_of_gets, total_nodes_usage.number_of_gets);
		node_reward.gets = ratio * total_customer_charge.gets;

		Some(node_reward)
	}

	fn get_customer_charge<T: Config>(
		cluster_id: ClusterId,
		usage: &CustomerUsage,
		start_era: i64,
		end_era: i64,
	) -> Result<CustomerCharge, Error<T>> {
		let mut total = CustomerCharge::default();

		let pricing = T::ClusterProtocol::get_pricing_params(&cluster_id)
			.map_err(|_| Error::<T>::NotExpectedClusterState)?;

		total.transfer = (|| -> Option<u128> {
			(usage.transferred_bytes as u128)
				.checked_mul(pricing.unit_per_mb_streamed)?
				.checked_div(byte_unit::MEBIBYTE)
		})()
		.ok_or(Error::<T>::ArithmeticOverflow)?;

		// Calculate the duration of the period in seconds
		let duration_seconds = end_era - start_era;
		let seconds_in_month = 30.44 * 24.0 * 3600.0;
		let fraction_of_month =
			Perquintill::from_rational(duration_seconds as u64, seconds_in_month as u64);

		total.storage = fraction_of_month *
			(|| -> Option<u128> {
				(usage.stored_bytes as u128)
					.checked_mul(pricing.unit_per_mb_stored)?
					.checked_div(byte_unit::MEBIBYTE)
			})()
			.ok_or(Error::<T>::ArithmeticOverflow)?;

		total.gets = (usage.number_of_gets as u128)
			.checked_mul(pricing.unit_per_get_request)
			.ok_or(Error::<T>::ArithmeticOverflow)?;

		total.puts = (usage.number_of_puts as u128)
			.checked_mul(pricing.unit_per_put_request)
			.ok_or(Error::<T>::ArithmeticOverflow)?;

		Ok(total)
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub feeder_account: Option<T::AccountId>,
		pub debtor_customers: Vec<(ClusterId, T::AccountId, u128)>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { feeder_account: None, debtor_customers: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			let account_id = <Pallet<T>>::account_id();
			let min = <T as pallet::Config>::Currency::minimum_balance();
			let balance = <T as pallet::Config>::Currency::free_balance(&account_id);
			if balance < min {
				if let Some(vault) = &self.feeder_account {
					let _ = <T as pallet::Config>::Currency::transfer(
						vault,
						&account_id,
						min - balance,
						ExistenceRequirement::AllowDeath,
					);
				} else {
					let _ = <T as pallet::Config>::Currency::make_free_balance_be(&account_id, min);
				}
			}

			for (cluster_id, customer_id, debt) in &self.debtor_customers {
				DebtorCustomers::<T>::insert(cluster_id, customer_id, debt);
			}
		}
	}

	impl<T: Config> PayoutVisitor<T> for Pallet<T> {
		fn begin_billing_report(
			origin: T::AccountId,
			cluster_id: ClusterId,
			era_id: DdcEra,
			start_era: i64,
			end_era: i64,
		) -> DispatchResult {
			let origin = frame_system::RawOrigin::Signed(origin).into();
			Self::begin_billing_report(origin, cluster_id, era_id, start_era, end_era)
		}

		fn begin_charging_customers(
			origin: T::AccountId,
			cluster_id: ClusterId,
			era_id: DdcEra,
			max_batch_index: BatchIndex,
		) -> DispatchResult {
			let origin = frame_system::RawOrigin::Signed(origin).into();
			Self::begin_charging_customers(origin, cluster_id, era_id, max_batch_index)
		}

		fn send_charging_customers_batch(
			origin: T::AccountId,
			cluster_id: ClusterId,
			era_id: DdcEra,
			batch_index: BatchIndex,
			payers: &[(T::AccountId, BucketId, CustomerUsage)],
			batch_proof: MMRProof,
		) -> DispatchResult {
			let origin = frame_system::RawOrigin::Signed(origin).into();
			Self::send_charging_customers_batch(
				origin,
				cluster_id,
				era_id,
				batch_index,
				(*payers).to_vec(),
				batch_proof,
			)
		}

		fn end_charging_customers(
			origin: T::AccountId,
			cluster_id: ClusterId,
			era_id: DdcEra,
		) -> DispatchResult {
			let origin = frame_system::RawOrigin::Signed(origin).into();
			Self::end_charging_customers(origin, cluster_id, era_id)
		}

		fn begin_rewarding_providers(
			origin: T::AccountId,
			cluster_id: ClusterId,
			era_id: DdcEra,
			max_batch_index: BatchIndex,
			total_node_usage: NodeUsage,
		) -> DispatchResult {
			let origin = frame_system::RawOrigin::Signed(origin).into();
			Self::begin_rewarding_providers(
				origin,
				cluster_id,
				era_id,
				max_batch_index,
				total_node_usage,
			)
		}

		fn send_rewarding_providers_batch(
			origin: T::AccountId,
			cluster_id: ClusterId,
			era_id: DdcEra,
			batch_index: BatchIndex,
			payees: &[(T::AccountId, BucketId, NodeUsage)],
			batch_proof: MMRProof,
		) -> DispatchResult {
			let origin = frame_system::RawOrigin::Signed(origin).into();
			Self::send_rewarding_providers_batch(
				origin,
				cluster_id,
				era_id,
				batch_index,
				(*payees).to_vec(),
				batch_proof,
			)
		}

		fn end_rewarding_providers(
			origin: T::AccountId,
			cluster_id: ClusterId,
			era_id: DdcEra,
		) -> DispatchResult {
			let origin = frame_system::RawOrigin::Signed(origin).into();
			Self::end_rewarding_providers(origin, cluster_id, era_id)
		}

		fn end_billing_report(
			origin: T::AccountId,
			cluster_id: ClusterId,
			era_id: DdcEra,
		) -> DispatchResult {
			let origin = frame_system::RawOrigin::Signed(origin).into();
			Self::end_billing_report(origin, cluster_id, era_id)
		}

		fn get_billing_report_status(cluster_id: &ClusterId, era: DdcEra) -> PayoutState {
			let billing_report = ActiveBillingReports::<T>::get(cluster_id, era);

			match billing_report {
				Some(report) => report.state,
				None => PayoutState::NotInitialized, // Return NotInitialized if entry doesn't exist
			}
		}

		fn all_customer_batches_processed(cluster_id: &ClusterId, era_id: DdcEra) -> bool {
			let billing_report = match ActiveBillingReports::<T>::try_get(cluster_id, era_id) {
				Ok(report) => report,
				Err(_) => return false, /* Return false if there's any error (e.g.,
				                         * BillingReportDoesNotExist) */
			};

			Self::validate_batches(
				&billing_report.charging_processed_batches,
				&billing_report.charging_max_batch_index,
			)
			.is_ok()
		}

		fn all_provider_batches_processed(cluster_id: &ClusterId, era_id: DdcEra) -> bool {
			let billing_report = match ActiveBillingReports::<T>::try_get(cluster_id, era_id) {
				Ok(report) => report,
				Err(_) => return false, /* Return false if there's any error (e.g.,
				                         * BillingReportDoesNotExist) */
			};

			Self::validate_batches(
				&billing_report.rewarding_processed_batches,
				&billing_report.rewarding_max_batch_index,
			)
			.is_ok()
		}

		fn get_next_customer_batch_for_payment(
			cluster_id: &ClusterId,
			era_id: DdcEra,
		) -> Result<Option<BatchIndex>, PayoutError> {
			let billing_report = ActiveBillingReports::<T>::try_get(cluster_id, era_id)
				.map_err(|_| PayoutError::BillingReportDoesNotExist)?;

			for batch_index in 0..=billing_report.charging_max_batch_index {
				if !billing_report.charging_processed_batches.contains(&batch_index) {
					return Ok(Some(batch_index));
				}
			}

			Ok(None)
		}

		fn get_next_provider_batch_for_payment(
			cluster_id: &ClusterId,
			era_id: DdcEra,
		) -> Result<Option<BatchIndex>, PayoutError> {
			let billing_report = ActiveBillingReports::<T>::try_get(cluster_id, era_id)
				.map_err(|_| PayoutError::BillingReportDoesNotExist)?;

			for batch_index in 0..=billing_report.rewarding_max_batch_index {
				if !billing_report.rewarding_processed_batches.contains(&batch_index) {
					return Ok(Some(batch_index));
				}
			}

			Ok(None)
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn sub_account_id(cluster_id: ClusterId, era: DdcEra) -> T::AccountId {
			let mut bytes = Vec::new();
			bytes.extend_from_slice(&cluster_id[..]);
			bytes.extend_from_slice(&era.encode());
			let hash = blake2_128(&bytes);

			// "modl" + "payouts_" + hash is 28 bytes, the T::AccountId is 32 bytes, so we should be
			// safe from the truncation and possible collisions caused by it. The rest 4 bytes will
			// be fulfilled with trailing zeros.
			T::PalletId::get().into_sub_account_truncating(hash)
		}

		pub(crate) fn validate_batches(
			batches: &BoundedBTreeSet<BatchIndex, MaxBatchesCount>,
			max_batch_index: &BatchIndex,
		) -> DispatchResult {
			// Check if the Vec contains all integers between 1 and rewarding_max_batch_index
			ensure!(!batches.is_empty(), Error::<T>::BatchesMissed);

			ensure!((*max_batch_index + 1) as usize == batches.len(), Error::<T>::BatchesMissed);

			for index in 0..*max_batch_index + 1 {
				ensure!(batches.contains(&index), Error::<T>::BatchesMissed);
			}

			Ok(())
		}
	}
}

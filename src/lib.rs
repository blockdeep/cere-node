//! # DDC Validator pallet
//!
//! The DDC Validator pallet defines storage item to store validation results and implements OCW
//! (off-chain worker) to produce these results using the data from Data Activity Capture (DAC).
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//! - [`Hooks`]
//!
//!	## Notes
//!
//! - Era definition in this pallet is different than in the `pallet-staking`. Check DAC
//!   documentation for `era` definition used in this pallet.

#![cfg_attr(not(feature = "std"), no_std)]

mod dac;
mod utils;
mod validation;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use alloc::{format, string::String};
pub use alt_serde::{de::DeserializeOwned, Deserialize, Serialize};
pub use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
pub use core::fmt::Debug;
pub use frame_support::{
	decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	pallet_prelude::*,
	parameter_types, storage,
	traits::{Currency, Randomness, UnixTime},
	weights::Weight,
	BoundedVec, RuntimeDebug,
};
pub use frame_system::{
	ensure_signed,
	offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer, SigningTypes},
	pallet_prelude::*,
};
pub use lite_json::json::JsonValue;
pub use pallet::*;
pub use pallet_ddc_staking::{self as ddc_staking};
pub use pallet_session as session;
pub use pallet_staking::{self as staking};
pub use scale_info::TypeInfo;
pub use serde_json::Value;
pub use sp_core::crypto::{AccountId32, KeyTypeId, UncheckedFrom};
pub use sp_io::crypto::sr25519_public_keys;
pub use sp_runtime::offchain::{http, storage::StorageValueRef, Duration, Timestamp};
pub use sp_staking::EraIndex;
pub use sp_std::{collections::btree_map::BTreeMap, prelude::*};

extern crate alloc;

/// The balance type of this pallet.
type BalanceOf<T> = <<T as pallet_contracts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

type ResultStr<T> = Result<T, &'static str>;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"dacv");

pub const TIME_START_MS: u128 = 1_672_531_200_000;
pub const ERA_DURATION_MS: u128 = 120_000;
pub const ERA_IN_BLOCKS: u8 = 20;

/// Webdis in experimental cluster connected to Redis in dev.
// pub const DEFAULT_DATA_PROVIDER_URL: &str = "https://dev-dac-redis.network-dev.aws.cere.io";
pub const DEFAULT_DATA_PROVIDER_URL: &str = "http://161.35.140.182:7379";
pub const DATA_PROVIDER_URL_KEY: &[u8; 32] = b"ddc-validator::data-provider-url";

/// Aggregated values from DAC that describe CDN node's activity during a certain era.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct DacTotalAggregates {
	/// Total bytes received by the client.
	pub received: u64,
	/// Total bytes sent by the CDN node.
	pub sent: u64,
	/// Total bytes sent by the CDN node to the client which interrupts the connection.
	pub failed_by_client: u64,
	/// ToDo: explain.
	pub failure_rate: u64,
}

/// Final DAC Validation decision.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ValidationDecision {
	/// Validation result.
	pub result: bool,
	/// A hash of the data used to produce validation result.
	pub payload: [u8; 256],
	/// Values aggregated from the payload.
	pub totals: DacTotalAggregates,
}

pub mod crypto {
	use super::KEY_TYPE;
	use frame_system::offchain::AppCrypto;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
	};
	app_crypto!(sr25519, KEY_TYPE);

	use sp_runtime::{MultiSignature, MultiSigner};

	pub struct TestAuthId;

	impl AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	impl AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_contracts::Config
		+ pallet_session::Config<ValidatorId = <Self as frame_system::Config>::AccountId>
		+ pallet_staking::Config
		+ ddc_staking::Config
		+ CreateSignedTransaction<Call<Self>>
	where
		<Self as frame_system::Config>::AccountId: AsRef<[u8]> + UncheckedFrom<Self::Hash>,
		<BalanceOf<Self> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
	{
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Something that provides randomness in the runtime. Required by the tasks assignment
		/// procedure.
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// A dispatchable call.
		type Call: From<Call<Self>>;

		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
		type TimeProvider: UnixTime;

		/// Number of validators expected to produce an individual validation decision to form a
		/// consensus. Tasks assignment procedure use this value to determine the number of
		/// validators are getting the same task. Must be an odd number.
		#[pallet::constant]
		type DdcValidatorsQuorumSize: Get<u32>;

		type ValidatorsMax: Get<u32>;

		/// Proof-of-Delivery parameter specifies an allowed deviation between bytes sent and bytes
		/// received. The deviation is expressed as a percentage. For example, if the value is 10,
		/// then the difference between bytes sent and bytes received is allowed to be up to 10%.
		/// The value must be in range [0, 100].
		#[pallet::constant]
		type ValidationThreshold: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn assignments)]
	pub(super) type Assignments<T: Config> =
		StorageDoubleMap<_, Twox64Concat, EraIndex, Twox64Concat, T::AccountId, Vec<T::AccountId>>;

	/// A signal to start a process on all the validators.
	#[pallet::storage]
	#[pallet::getter(fn signal)]
	pub(super) type Signal<T: Config> = StorageValue<_, bool>;

	/// The map from the era and CDN participant stash key to the validation decision related.
	#[pallet::storage]
	#[pallet::getter(fn validation_decisions)]
	pub type ValidationDecisions<T: Config> =
		StorageDoubleMap<_, Twox64Concat, EraIndex, Twox64Concat, T::AccountId, ValidationDecision>;

	/// The last era for which the tasks assignment produced.
	#[pallet::storage]
	#[pallet::getter(fn last_managed_era)]
	pub type LastManagedEra<T: Config> = StorageValue<_, EraIndex>;

	#[pallet::error]
	pub enum Error<T> {
		// TBA
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>
	where
		<T as frame_system::Config>::AccountId: AsRef<[u8]> + UncheckedFrom<T::Hash>,
		<BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode, {}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		<T as frame_system::Config>::AccountId: AsRef<[u8]> + UncheckedFrom<T::Hash>,
		<BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
	{
		fn on_initialize(block_number: T::BlockNumber) -> Weight {
			if block_number <= 1u32.into() {
				return 0
			}

			let era = Self::get_current_era();
			log::info!("current era: {:?}", era);

			if let Some(last_managed_era) = <LastManagedEra<T>>::get() {
				log::info!("last_managed_era: {:?}", last_managed_era);
				if last_managed_era >= era {
					return 0
				}
			}
			<LastManagedEra<T>>::put(era);

			Self::assign(3usize);

			0
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			// Skip if not a validator.
			if !sp_io::offchain::is_validator() {
				return
			}

			let data_provider_url = Self::get_data_provider_url();
			log::info!("[DAC Validator] Data provider URL: {:?}", &data_provider_url);

			let mock_data_url = Self::get_mock_data_url();

			let file_request = dac::fetch_file_request(&mock_data_url);
			let bytes_sum = dac::get_proved_delivered_bytes_sum(&file_request);
			log::info!("Proved bytes sum: {:?}", bytes_sum);

			// Print the number of broken sessions per CDN node.
			let aggregates_value = dac::fetch_aggregates(&data_provider_url, 77436).unwrap(); // 77436 is for a mock data
			let aggregates_obj = aggregates_value.as_object().unwrap();
			aggregates_obj
				.into_iter()
				.for_each(|(cdn_node_pubkey, cdn_node_aggregates_value)| {
					// iterate over aggregates for each node
					let cdn_node_aggregates_obj = cdn_node_aggregates_value.as_object().unwrap();
					// Extract `nodeInterruptedSessions` field
					let (_, cdn_node_interrupted_sessions_value) = cdn_node_aggregates_obj
						.into_iter()
						.find(|(key, _)| key.iter().copied().eq("nodeInterruptedSessions".chars()))
						.unwrap();
					let cdn_node_interrupted_sessions_obj =
						cdn_node_interrupted_sessions_value.as_object().unwrap();
					// Prepare CDN pubkey without heap allocated string
					let cdn_node_pubkey_vecu8: Vec<u8> =
						cdn_node_pubkey.iter().map(|c| *c as u8).collect();
					let cdn_node_pubkey_str =
						sp_std::str::from_utf8(&cdn_node_pubkey_vecu8).unwrap();
					log::info!(
						"Broken sessions per CDN node | Node {}: {} sessions broken",
						cdn_node_pubkey_str,
						cdn_node_interrupted_sessions_obj.len(), /* count sessions broken by the
						                                          * node */
					);
				});

			// Wait for signal.
			let signal = Signal::<T>::get().unwrap_or(false);
			if !signal {
				log::info!("🔎 DAC Validator is idle at block {:?}, waiting for a signal, signal state is {:?}", block_number, signal);
				return
			}

			// Read from DAC.
			let current_era = Self::get_current_era();
			let (sent_query, sent, received_query, received) =
				dac::fetch_data2(&data_provider_url, current_era - 1);
			log::info!(
				"🔎 DAC Validator is fetching data from DAC, current era: {:?}, bytes sent query: {:?}, bytes sent response: {:?}, bytes received query: {:?}, bytes received response: {:?}",
				current_era,
				sent_query,
				sent,
				received_query,
				received,
			);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as frame_system::Config>::AccountId: AsRef<[u8]> + UncheckedFrom<T::Hash>,
		<BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
	{
		/// Run a process at the same time on all the validators.
		#[pallet::weight(10_000)]
		pub fn send_signal(origin: OriginFor<T>) -> DispatchResult {
			ensure_signed(origin)?;

			Signal::<T>::set(Some(true));

			Ok(())
		}

		/// Set validation decision for a given CDN node in an era.
		#[pallet::weight(10_000)]
		pub fn set_validation_decision(
			origin: OriginFor<T>,
			era: EraIndex,
			cdn_node: T::AccountId,
			validation_decision: ValidationDecision,
		) -> DispatchResult {
			ensure_signed(origin)?;

			// ToDo: check if origin is a validator.
			// ToDo: check if the era is current - 1.
			// ToDo: check if the validation decision is not set yet.
			// ToDo: check cdn_node is known to ddc-staking.

			ValidationDecisions::<T>::insert(era, cdn_node, validation_decision);

			// ToDo: emit event.

			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		<T as frame_system::Config>::AccountId: AsRef<[u8]> + UncheckedFrom<T::Hash>,
		<BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
	{
		fn get_data_provider_url() -> String {
			let url_ref = sp_io::offchain::local_storage_get(
				sp_core::offchain::StorageKind::PERSISTENT,
				DATA_PROVIDER_URL_KEY,
			);

			match url_ref {
				Some(url) =>
					String::from_utf8(url).expect("Data provider URL should be valid UTF-8 string"),
				None => String::from(DEFAULT_DATA_PROVIDER_URL),
			}
		}

		fn get_mock_data_url() -> String {
			let data_url = Self::get_data_provider_url();
			let mock_url = "/JSON.GET/testddc:dac:data";
			let url = format!("{}{}", data_url, mock_url);

			url
		}

		fn get_signer() -> ResultStr<Signer<T, T::AuthorityId>> {
			let signer = Signer::<_, _>::any_account();
			if !signer.can_sign() {
				return Err("[DAC Validator] No local accounts available. Consider adding one via `author_insertKey` RPC.");
			}

			Ok(signer)
		}

		// Get the current era; Shall we start era count from 0 or from 1?
		fn get_current_era() -> EraIndex {
			((T::TimeProvider::now().as_millis() - TIME_START_MS) / ERA_DURATION_MS)
				.try_into()
				.unwrap()
		}

		fn validate(bytes_sent: dac::BytesSent, bytes_received: dac::BytesReceived) -> bool {
			let percentage_difference = 1f32 - (bytes_received.sum as f32 / bytes_sent.sum as f32);

			return if percentage_difference > 0.0 &&
				(T::ValidationThreshold::get() as f32 - percentage_difference) > 0.0
			{
				true
			} else {
				false
			}
		}

		fn shuffle(mut list: Vec<T::AccountId>) -> Vec<T::AccountId> {
			let len = list.len();
			for i in 1..len {
				let random_index = Self::choose(len as u32).unwrap() as usize;
				list.swap(i, random_index)
			}

			list
		}

		fn split<Item: Clone>(list: Vec<Item>, segment_len: usize) -> Vec<Vec<Item>> {
			list.chunks(segment_len).map(|chunk| chunk.to_vec()).collect()
		}

		fn assign(quorum_size: usize) {
			let validators: Vec<T::AccountId> = <staking::Validators<T>>::iter_keys().collect();
			let edges: Vec<T::AccountId> = <ddc_staking::pallet::Edges<T>>::iter_keys().collect();

			if edges.len() == 0 {
				return
			}

			let shuffled_validators = Self::shuffle(validators);
			let shuffled_edges = Self::shuffle(edges);

			let validators_keys: Vec<String> = shuffled_validators
				.iter()
				.map(|v| utils::account_to_string::<T>(v.clone()))
				.collect();

			let quorums = Self::split(validators_keys, quorum_size);
			let edges_groups = Self::split(shuffled_edges, quorums.len());

			let era = Self::get_current_era();

			for (i, quorum) in quorums.iter().enumerate() {
				let edges_group = &edges_groups[i];
				for validator in quorum {
					Assignments::<T>::insert(
						era,
						utils::string_to_account::<T>(validator.clone()),
						edges_group,
					);
				}
			}
		}

		/// Randomly choose a number in range `[0, total)`.
		/// Returns `None` for zero input.
		/// Modification of `choose_ticket` from `pallet-lottery` version `4.0.0-dev`.
		fn choose(total: u32) -> Option<u32> {
			if total == 0 {
				return None
			}
			let mut random_number = Self::generate_random_number(0);

			// Best effort attempt to remove bias from modulus operator.
			for i in 1..128 {
				if random_number < u32::MAX - u32::MAX % total {
					break
				}

				random_number = Self::generate_random_number(i);
			}

			Some(random_number % total)
		}

		/// Generate a random number from a given seed.
		/// Note that there is potential bias introduced by using modulus operator.
		/// You should call this function with different seed values until the random
		/// number lies within `u32::MAX - u32::MAX % n`.
		/// Modification of `generate_random_number` from `pallet-lottery` version `4.0.0-dev`.
		fn generate_random_number(seed: u32) -> u32 {
			let (random_seed, _) =
				<T as pallet::Config>::Randomness::random(&(b"ddc-validator", seed).encode());
			let random_number = <u32>::decode(&mut random_seed.as_ref())
				.expect("secure hashes should always be bigger than u32; qed");

			random_number
		}
	}
}

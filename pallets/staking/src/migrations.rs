// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and

//! Storage migrations for the Staking pallet.

use frame_election_provider_support::SortedListProvider;
#[cfg(feature = "try-runtime")]
use frame_support::ensure;
use frame_support::{
	dispatch::GetStorageVersion, pallet_prelude::ValueQuery, storage_alias,
	traits::OnRuntimeUpgrade,
};
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

use super::*;

/// Used for release versioning upto v12.
///
/// Obsolete from v13. Keeping around to make encoding/decoding of old migration code easier.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
enum ObsoleteReleases {
	V1_0_0Ancient,
	V2_0_0,
	V3_0_0,
	V4_0_0,
	V5_0_0,  // blockable validators.
	V6_0_0,  // removal of all storage associated with offchain phragmen.
	V7_0_0,  // keep track of number of nominators / validators in map
	V8_0_0,  // populate `VoterList`.
	V9_0_0,  // inject validators into `VoterList` as well.
	V10_0_0, // remove `EarliestUnappliedSlash`.
	V11_0_0, // Move pallet storage prefix, e.g. BagsList -> VoterBagsList
	V12_0_0, // remove `HistoryDepth`.
}

impl Default for ObsoleteReleases {
	fn default() -> Self {
		ObsoleteReleases::V12_0_0
	}
}

/// Alias to the old storage item used for release versioning. Obsolete since v13.
#[storage_alias]
type StorageVersion<T: Config> = StorageValue<Pallet<T>, ObsoleteReleases, ValueQuery>;

pub mod v13 {
	use super::*;

	pub struct MigrateToV13<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV13<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			frame_support::ensure!(
				StorageVersion::<T>::get() == ObsoleteReleases::V12_0_0,
				"Required v12 before upgrading to v13"
			);

			Ok(Default::default())
		}

		fn on_runtime_upgrade() -> Weight {
			let current = Pallet::<T>::current_storage_version();
			let onchain = StorageVersion::<T>::get();

			if current == 13 && onchain == ObsoleteReleases::V12_0_0 {
				StorageVersion::<T>::kill();
				current.put::<Pallet<T>>();

				log!(info, "v13 applied successfully");
				T::DbWeight::get().reads_writes(1, 2)
			} else {
				log!(warn, "Skipping v13, should be removed");
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			frame_support::ensure!(
				Pallet::<T>::on_chain_storage_version() == 13,
				"v13 not applied"
			);

			frame_support::ensure!(
				!StorageVersion::<T>::exists(),
				"Storage version not migrated correctly"
			);

			Ok(())
		}
	}
}

pub mod v12 {
	use frame_support::{pallet_prelude::ValueQuery, storage_alias};

	use super::*;

	#[storage_alias]
	type HistoryDepth<T: Config> = StorageValue<Pallet<T>, u32, ValueQuery>;

	/// Clean up `HistoryDepth` from storage.
	///
	/// We will be depending on the configurable value of `HistoryDepth` post
	/// this release.
	pub struct MigrateToV12<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV12<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			frame_support::ensure!(
				StorageVersion::<T>::get() == ObsoleteReleases::V11_0_0,
				"Expected v11 before upgrading to v12"
			);

			if HistoryDepth::<T>::exists() {
				frame_support::ensure!(
					T::HistoryDepth::get() == HistoryDepth::<T>::get(),
					"Provided value of HistoryDepth should be same as the existing storage value"
				);
			} else {
				log::info!("No HistoryDepth in storage; nothing to remove");
			}

			Ok(Default::default())
		}

		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			if StorageVersion::<T>::get() == ObsoleteReleases::V11_0_0 {
				HistoryDepth::<T>::kill();
				StorageVersion::<T>::put(ObsoleteReleases::V12_0_0);

				log!(info, "v12 applied successfully");
				T::DbWeight::get().reads_writes(1, 2)
			} else {
				log!(warn, "Skipping v12, should be removed");
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			frame_support::ensure!(
				StorageVersion::<T>::get() == ObsoleteReleases::V12_0_0,
				"v12 not applied"
			);
			Ok(())
		}
	}
}

pub mod v11 {
	use frame_support::{
		storage::migration::move_pallet,
		traits::{GetStorageVersion, PalletInfoAccess},
	};
	#[cfg(feature = "try-runtime")]
	use sp_io::hashing::twox_128;

	use super::*;

	pub struct MigrateToV11<T, P, N>(sp_std::marker::PhantomData<(T, P, N)>);
	impl<T: Config, P: GetStorageVersion + PalletInfoAccess, N: Get<&'static str>> OnRuntimeUpgrade
		for MigrateToV11<T, P, N>
	{
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			frame_support::ensure!(
				StorageVersion::<T>::get() == ObsoleteReleases::V10_0_0,
				"must upgrade linearly"
			);
			let old_pallet_prefix = twox_128(N::get().as_bytes());

			frame_support::ensure!(
				sp_io::storage::next_key(&old_pallet_prefix).is_some(),
				"no data for the old pallet name has been detected"
			);

			Ok(Default::default())
		}

		/// Migrate the entire storage of this pallet to a new prefix.
		///
		/// Since Cere is already using VoterList, we just need to remove the old prefix
		///
		/// This new prefix must be the same as the one set in construct_runtime. For safety, use
		/// `PalletInfo` to get it, as:
		/// `<Runtime as frame_system::Config>::PalletInfo::name::<VoterBagsList>`.
		///
		/// The migration will look into the storage version in order to avoid triggering a
		/// migration on an up to date storage.
		fn on_runtime_upgrade() -> Weight {
			if StorageVersion::<T>::get() == ObsoleteReleases::V10_0_0 {
				log!(info, "removing {}", N::get());

				let _ = frame_support::storage::migration::clear_storage_prefix(
					N::get().as_bytes(),
					&[],
					&[],
					None,
					None,
				);

				StorageVersion::<T>::put(ObsoleteReleases::V11_0_0);

				<T as frame_system::Config>::BlockWeights::get().max_block
			} else {
				log!(warn, "v11::migrate should be removed.");
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			frame_support::ensure!(
				StorageVersion::<T>::get() == ObsoleteReleases::V11_0_0,
				"wrong version after the upgrade"
			);

			let old_pallet_name = N::get();
			let new_pallet_name = <P as PalletInfoAccess>::name();

			// skip storage prefix checks for the same pallet names
			if new_pallet_name == old_pallet_name {
				return Ok(())
			}

			let old_pallet_prefix = twox_128(N::get().as_bytes());
			frame_support::ensure!(
				sp_io::storage::next_key(&old_pallet_prefix).is_none(),
				"old pallet data hasn't been removed"
			);

			let new_pallet_name = <P as PalletInfoAccess>::name();
			let new_pallet_prefix = twox_128(new_pallet_name.as_bytes());
			frame_support::ensure!(
				sp_io::storage::next_key(&new_pallet_prefix).is_some(),
				"new pallet data hasn't been created"
			);

			Ok(())
		}
	}
}

pub mod v10 {
	use frame_support::storage_alias;

	use super::*;

	#[storage_alias]
	type EarliestUnappliedSlash<T: Config> = StorageValue<Pallet<T>, EraIndex>;

	/// Apply any pending slashes that where queued.
	///
	/// That means we might slash someone a bit too early, but we will definitely
	/// won't forget to slash them. The cap of 512 is somewhat randomly taken to
	/// prevent us from iterating over an arbitrary large number of keys `on_runtime_upgrade`.
	pub struct MigrateToV10<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV10<T> {
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			if StorageVersion::<T>::get() == ObsoleteReleases::V9_0_0 {
				let pending_slashes = UnappliedSlashes::<T>::iter().take(512);
				for (era, slashes) in pending_slashes {
					for slash in slashes {
						// in the old slashing scheme, the slash era was the key at which we read
						// from `UnappliedSlashes`.
						log!(warn, "prematurely applying a slash ({:?}) for era {:?}", slash, era);
						slashing::apply_slash::<T>(slash, era);
					}
				}

				EarliestUnappliedSlash::<T>::kill();
				StorageVersion::<T>::put(ObsoleteReleases::V10_0_0);

				log!(info, "MigrateToV10 executed successfully");
				T::DbWeight::get().reads_writes(1, 1)
			} else {
				log!(warn, "MigrateToV10 should be removed.");
				T::DbWeight::get().reads(1)
			}
		}
	}
}

pub mod v9 {
	#[cfg(feature = "try-runtime")]
	use frame_support::codec::{Decode, Encode};
	#[cfg(feature = "try-runtime")]
	use sp_std::vec::Vec;

	use super::*;

	/// Migration implementation that injects all validators into sorted list.
	///
	/// Skip the migration and update only StorageVersion since Cere already has all
	/// validators in the list
	///
	/// This is only useful for chains that started their `VoterList` just based on nominators.
	pub struct InjectValidatorsIntoVoterList<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for InjectValidatorsIntoVoterList<T> {
		fn on_runtime_upgrade() -> Weight {
			if StorageVersion::<T>::get() == ObsoleteReleases::V8_0_0 {
				log!(info, "migrating staking to ObsoleteReleases::V9_0_0");

				StorageVersion::<T>::put(ObsoleteReleases::V9_0_0);
				T::DbWeight::get().reads_writes(1, 1)
			} else {
				log!(
					warn,
					"InjectValidatorsIntoVoterList being executed on the wrong storage \
				version, expected ObsoleteReleases::V8_0_0"
				);
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			frame_support::ensure!(
				StorageVersion::<T>::get() == ObsoleteReleases::V8_0_0,
				"must upgrade linearly"
			);

			let prev_count = T::VoterList::count();
			Ok(prev_count.encode())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(prev_count: Vec<u8>) -> Result<(), TryRuntimeError> {
			let prev_count: u32 = Decode::decode(&mut prev_count.as_slice()).expect(
				"the state parameter should be something that was generated by pre_upgrade",
			);
			let post_count = T::VoterList::count();
			ensure!(
				post_count == prev_count,
				"`VoterList` count after the migration must equal to the sum of \
				previous count since we just skipping the migration"
			);

			frame_support::ensure!(
				StorageVersion::<T>::get() == ObsoleteReleases::V9_0_0,
				"must upgrade"
			);
			Ok(())
		}
	}
}

pub mod v8 {
	use frame_election_provider_support::SortedListProvider;
	use frame_support::traits::Get;

	use super::*;
	use crate::{Config, Nominators, Pallet, Weight};

	#[cfg(feature = "try-runtime")]
	pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
		frame_support::ensure!(
			StorageVersion::<T>::get() == ObsoleteReleases::V7_0_0,
			"must upgrade linearly"
		);

		crate::log!(info, "👜 staking bags-list migration passes PRE migrate checks ✅",);
		Ok(())
	}

	/// Migration to sorted `VoterList`.
	pub fn migrate<T: Config>() -> Weight {
		if StorageVersion::<T>::get() == ObsoleteReleases::V7_0_0 {
			crate::log!(info, "migrating staking to ObsoleteReleases::V8_0_0");

			let migrated = T::VoterList::unsafe_regenerate(
				Nominators::<T>::iter().map(|(id, _)| id),
				Pallet::<T>::weight_of_fn(),
			);

			StorageVersion::<T>::put(ObsoleteReleases::V8_0_0);
			crate::log!(
				info,
				"👜 completed staking migration to ObsoleteReleases::V8_0_0 with {} voters migrated",
				migrated,
			);

			T::BlockWeights::get().max_block
		} else {
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
		T::VoterList::try_state().map_err(|_| "VoterList is not in a sane state.")?;
		crate::log!(info, "👜 staking bags-list migration passes POST migrate checks ✅",);
		Ok(())
	}
}

pub mod v7 {
	use frame_support::storage_alias;

	use super::*;

	#[storage_alias]
	type CounterForValidators<T: Config> = StorageValue<Pallet<T>, u32>;
	#[storage_alias]
	type CounterForNominators<T: Config> = StorageValue<Pallet<T>, u32>;

	pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
		assert!(
			CounterForValidators::<T>::get().unwrap().is_zero(),
			"CounterForValidators already set."
		);
		assert!(
			CounterForNominators::<T>::get().unwrap().is_zero(),
			"CounterForNominators already set."
		);
		assert!(Validators::<T>::count().is_zero(), "Validators already set.");
		assert!(Nominators::<T>::count().is_zero(), "Nominators already set.");
		assert!(StorageVersion::<T>::get() == ObsoleteReleases::V6_0_0);
		Ok(())
	}

	pub fn migrate<T: Config>() -> Weight {
		log!(info, "Migrating staking to ObsoleteReleases::V7_0_0");
		let validator_count = Validators::<T>::iter().count() as u32;
		let nominator_count = Nominators::<T>::iter().count() as u32;

		CounterForValidators::<T>::put(validator_count);
		CounterForNominators::<T>::put(nominator_count);

		StorageVersion::<T>::put(ObsoleteReleases::V7_0_0);
		log!(info, "Completed staking migration to ObsoleteReleases::V7_0_0");

		T::DbWeight::get().reads_writes(validator_count.saturating_add(nominator_count).into(), 2)
	}
}

pub mod v6 {
	use frame_support::{storage_alias, traits::Get, weights::Weight};

	use super::*;

	// NOTE: value type doesn't matter, we just set it to () here.
	#[storage_alias]
	type SnapshotValidators<T: Config> = StorageValue<Pallet<T>, ()>;
	#[storage_alias]
	type SnapshotNominators<T: Config> = StorageValue<Pallet<T>, ()>;
	#[storage_alias]
	type QueuedElected<T: Config> = StorageValue<Pallet<T>, ()>;
	#[storage_alias]
	type QueuedScore<T: Config> = StorageValue<Pallet<T>, ()>;
	#[storage_alias]
	type EraElectionStatus<T: Config> = StorageValue<Pallet<T>, ()>;
	#[storage_alias]
	type IsCurrentSessionFinal<T: Config> = StorageValue<Pallet<T>, ()>;

	/// check to execute prior to migration.
	pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
		// these may or may not exist.
		log!(info, "SnapshotValidators.exits()? {:?}", SnapshotValidators::<T>::exists());
		log!(info, "SnapshotNominators.exits()? {:?}", SnapshotNominators::<T>::exists());
		log!(info, "QueuedElected.exits()? {:?}", QueuedElected::<T>::exists());
		log!(info, "QueuedScore.exits()? {:?}", QueuedScore::<T>::exists());
		// these must exist.
		assert!(
			IsCurrentSessionFinal::<T>::exists(),
			"IsCurrentSessionFinal storage item not found!"
		);
		assert!(EraElectionStatus::<T>::exists(), "EraElectionStatus storage item not found!");
		Ok(())
	}

	/// Migrate storage to v6.
	pub fn migrate<T: Config>() -> Weight {
		log!(info, "Migrating staking to ObsoleteReleases::V6_0_0");

		SnapshotValidators::<T>::kill();
		SnapshotNominators::<T>::kill();
		QueuedElected::<T>::kill();
		QueuedScore::<T>::kill();
		EraElectionStatus::<T>::kill();
		IsCurrentSessionFinal::<T>::kill();

		StorageVersion::<T>::put(ObsoleteReleases::V6_0_0);

		log!(info, "Done.");
		T::DbWeight::get().writes(6 + 1)
	}
}

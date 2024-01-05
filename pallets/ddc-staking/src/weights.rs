//! Autogenerated weights for pallet_ddc_staking
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-12-14, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Yahors-MacBook-Pro.local`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/cere
// benchmark
// pallet
// --chain=dev
// --execution=wasm
// --pallet=pallet-ddc-staking
// --extrinsic=*
// --steps=50
// --repeat=20
// --template=./.maintain/frame-weight-template.hbs
// --output=pallets/ddc-staking/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_ddc_staking.
pub trait WeightInfo {
	fn bond() -> Weight;
	fn unbond() -> Weight;
	fn withdraw_unbonded() -> Weight;
	fn store() -> Weight;
	fn chill() -> Weight;
	fn set_controller() -> Weight;
	fn set_node() -> Weight;
}

/// Weights for pallet_ddc_staking using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: DdcStaking Bonded (r:1 w:1)
	// Storage: DdcStaking Ledger (r:1 w:1)
	// Storage: DdcStaking Nodes (r:1 w:1)
	// Storage: DdcStaking Providers (r:1 w:1)
	// Storage: DdcNodes StorageNodes (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	fn bond() -> Weight {
		Weight::from_ref_time(39_000_000_u64)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	// Storage: DdcStaking Ledger (r:1 w:1)
	// Storage: DdcStaking Storages (r:1 w:0)
	// Storage: DdcStaking Providers (r:1 w:0)
	// Storage: DdcNodes StorageNodes (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn unbond() -> Weight {
		Weight::from_ref_time(37_000_000_u64)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	// Storage: DdcStaking Ledger (r:1 w:1)
	// Storage: DdcStaking Providers (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: DdcStaking LeavingStorages (r:1 w:0)
	fn withdraw_unbonded() -> Weight {
		Weight::from_ref_time(33_000_000_u64)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	// Storage: DdcClusters Clusters (r:1 w:0)
	// Storage: DdcStaking Ledger (r:1 w:0)
	// Storage: DdcClusters ClustersGovParams (r:1 w:0)
	// Storage: DdcStaking Providers (r:1 w:0)
	// Storage: DdcStaking Storages (r:1 w:1)
	// Storage: DdcStaking LeavingStorages (r:1 w:0)
	fn store() -> Weight {
		Weight::from_ref_time(28_000_000_u64)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: DdcStaking Ledger (r:1 w:1)
	// Storage: DdcStaking Storages (r:1 w:1)
	// Storage: DdcClusters ClustersGovParams (r:1 w:0)
	fn chill() -> Weight {
		Weight::from_ref_time(28_000_000_u64)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	// Storage: DdcStaking Bonded (r:1 w:1)
	// Storage: DdcStaking Ledger (r:2 w:2)
	fn set_controller() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	// Storage: DdcStaking Nodes (r:1 w:2)
	// Storage: DdcStaking Providers (r:1 w:1)
	// Storage: DdcStaking Storages (r:1 w:0)
	// Storage: DdcStaking LeavingStorages (r:1 w:0)
	fn set_node() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: DdcStaking Bonded (r:1 w:1)
	// Storage: DdcStaking Ledger (r:1 w:1)
	// Storage: DdcStaking Nodes (r:1 w:1)
	// Storage: DdcStaking Providers (r:1 w:1)
	// Storage: DdcNodes StorageNodes (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	fn bond() -> Weight {
		Weight::from_ref_time(39_000_000_u64)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	// Storage: DdcStaking Ledger (r:1 w:1)
	// Storage: DdcStaking Storages (r:1 w:0)
	// Storage: DdcStaking Providers (r:1 w:0)
	// Storage: DdcNodes StorageNodes (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn unbond() -> Weight {
		Weight::from_ref_time(37_000_000_u64)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	// Storage: DdcStaking Ledger (r:1 w:1)
	// Storage: DdcStaking Providers (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: DdcStaking LeavingStorages (r:1 w:0)
	fn withdraw_unbonded() -> Weight {
		Weight::from_ref_time(33_000_000_u64)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	// Storage: DdcClusters Clusters (r:1 w:0)
	// Storage: DdcStaking Ledger (r:1 w:0)
	// Storage: DdcClusters ClustersGovParams (r:1 w:0)
	// Storage: DdcStaking Providers (r:1 w:0)
	// Storage: DdcStaking Storages (r:1 w:1)
	// Storage: DdcStaking LeavingStorages (r:1 w:0)
	fn store() -> Weight {
		Weight::from_ref_time(28_000_000_u64)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: DdcStaking Ledger (r:1 w:1)
	// Storage: DdcStaking Storages (r:1 w:1)
	// Storage: DdcClusters ClustersGovParams (r:1 w:0)
	fn chill() -> Weight {
		Weight::from_ref_time(28_000_000_u64)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	// Storage: DdcStaking Bonded (r:1 w:1)
	// Storage: DdcStaking Ledger (r:2 w:2)
	fn set_controller() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	// Storage: DdcStaking Nodes (r:1 w:2)
	// Storage: DdcStaking Providers (r:1 w:1)
	// Storage: DdcStaking Storages (r:1 w:0)
	// Storage: DdcStaking LeavingStorages (r:1 w:0)
	fn set_node() -> Weight {
		Weight::from_ref_time(14_000_000_u64)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
}
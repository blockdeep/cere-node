//! Autogenerated weights for pallet_ddc_verification
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-05-21, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `192.168.1.4`, CPU: `<UNKNOWN>`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/cere
// benchmark
// pallet
// --chain
// dev
// --wasm-execution=compiled
// --pallet
// pallet_ddc_verification
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --output=./pallets/ddc-verification/src/weights.rs
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_ddc_verification.
pub trait WeightInfo {
	fn create_billing_reports() -> Weight;
}

/// Weights for pallet_ddc_verification using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: `DdcVerification::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcVerification::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn create_billing_reports() -> Weight {
		Weight::from_parts(11_000_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: `DdcVerification::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcVerification::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn create_billing_reports() -> Weight {
		Weight::from_parts(11_000_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}

//! Autogenerated weights for pallet_ddc_verification
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-12-04, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `bench`, CPU: `AMD EPYC Processor`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/cere
// benchmark
// pallet
// --chain=dev
// --execution=wasm
// --pallet=pallet_ddc_verification
// --extrinsic=*
// --steps=50
// --repeat=20
// --template=./.maintain/frame-weight-template.hbs
// --output=pallets/ddc-verification/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_ddc_verification.
pub trait WeightInfo {
	fn set_prepare_era_for_payout(b: u32, ) -> Weight;
	fn set_validator_key() -> Weight;
	fn commit_billing_fingerprint() -> Weight;
	fn begin_billing_report() -> Weight;
	fn begin_charging_customers() -> Weight;
	fn send_charging_customers_batch(b: u32, ) -> Weight;
	fn end_charging_customers() -> Weight;
	fn begin_rewarding_providers() -> Weight;
	fn send_rewarding_providers_batch(b: u32, ) -> Weight;
	fn end_rewarding_providers() -> Weight;
	fn end_billing_report() -> Weight;
	fn emit_consensus_errors(b: u32, ) -> Weight;
	fn set_era_validations() -> Weight;
	fn skip_dac_validation_to_era() -> Weight;
}

/// Weights for pallet_ddc_verification using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::EraValidations` (r:1 w:1)
	// Proof: `DdcVerification::EraValidations` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[1, 5]`.
	fn set_prepare_era_for_payout(b: u32, ) -> Weight {
		Weight::from_parts(33_337_691_u64, 0)
			// Standard Error: 12_689
			.saturating_add(Weight::from_parts(380_174_u64, 0).saturating_mul(b as u64))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: `Staking::Ledger` (r:1 w:0)
	// Proof: `Staking::Ledger` (`max_values`: None, `max_size`: Some(1091), added: 3566, mode: `MaxEncodedLen`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorToStashKey` (r:0 w:1)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_validator_key() -> Weight {
		Weight::from_parts(29_155_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcClusters::Clusters` (r:1 w:0)
	// Proof: `DdcClusters::Clusters` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::BillingFingerprints` (r:1 w:1)
	// Proof: `DdcPayouts::BillingFingerprints` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn commit_billing_fingerprint() -> Weight {
		Weight::from_parts(40_165_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::BillingFingerprints` (r:1 w:0)
	// Proof: `DdcPayouts::BillingFingerprints` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::EraValidations` (r:1 w:1)
	// Proof: `DdcVerification::EraValidations` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn begin_billing_report() -> Weight {
		Weight::from_parts(45_956_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn begin_charging_customers() -> Weight {
		Weight::from_parts(32_561_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::BillingFingerprints` (r:1 w:0)
	// Proof: `DdcPayouts::BillingFingerprints` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcClusters::ClustersGovParams` (r:1 w:0)
	// Proof: `DdcClusters::ClustersGovParams` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcCustomers::Buckets` (r:500 w:500)
	// Proof: `DdcCustomers::Buckets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcCustomers::Ledger` (r:500 w:500)
	// Proof: `DdcCustomers::Ledger` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `System::Account` (r:2 w:2)
	// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	// Storage: `DdcPayouts::DebtorCustomers` (r:489 w:489)
	// Proof: `DdcPayouts::DebtorCustomers` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[1, 500]`.
	fn send_charging_customers_batch(b: u32, ) -> Weight {
		Weight::from_parts(149_151_000_u64, 0)
			// Standard Error: 236_904
			.saturating_add(Weight::from_parts(80_520_850_u64, 0).saturating_mul(b as u64))
			.saturating_add(T::DbWeight::get().reads(10_u64))
			.saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(b as u64)))
			.saturating_add(T::DbWeight::get().writes(6_u64))
			.saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(b as u64)))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcClusters::ClustersGovParams` (r:1 w:0)
	// Proof: `DdcClusters::ClustersGovParams` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `System::Account` (r:3 w:3)
	// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	// Storage: `DdcClusters::Clusters` (r:1 w:0)
	// Proof: `DdcClusters::Clusters` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `Staking::Validators` (r:2 w:0)
	// Proof: `Staking::Validators` (`max_values`: None, `max_size`: Some(45), added: 2520, mode: `MaxEncodedLen`)
	// Storage: `Staking::Bonded` (r:1 w:0)
	// Proof: `Staking::Bonded` (`max_values`: None, `max_size`: Some(72), added: 2547, mode: `MaxEncodedLen`)
	// Storage: `Staking::Ledger` (r:1 w:0)
	// Proof: `Staking::Ledger` (`max_values`: None, `max_size`: Some(1091), added: 3566, mode: `MaxEncodedLen`)
	// Storage: `Staking::Nominators` (r:1 w:0)
	// Proof: `Staking::Nominators` (`max_values`: None, `max_size`: Some(558), added: 3033, mode: `MaxEncodedLen`)
	fn end_charging_customers() -> Weight {
		Weight::from_parts(250_869_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(13_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn begin_rewarding_providers() -> Weight {
		Weight::from_parts(33_362_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::BillingFingerprints` (r:1 w:0)
	// Proof: `DdcPayouts::BillingFingerprints` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcNodes::StorageNodes` (r:500 w:500)
	// Proof: `DdcNodes::StorageNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `System::Account` (r:501 w:501)
	// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[1, 500]`.
	fn send_rewarding_providers_batch(b: u32, ) -> Weight {
		Weight::from_parts(122_499_000_u64, 0)
			// Standard Error: 54_711
			.saturating_add(Weight::from_parts(76_199_725_u64, 0).saturating_mul(b as u64))
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(b as u64)))
			.saturating_add(T::DbWeight::get().writes(2_u64))
			.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(b as u64)))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn end_rewarding_providers() -> Weight {
		Weight::from_parts(34_094_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::EraValidations` (r:1 w:1)
	// Proof: `DdcVerification::EraValidations` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcClusters::Clusters` (r:1 w:1)
	// Proof: `DdcClusters::Clusters` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn end_billing_report() -> Weight {
		Weight::from_parts(62_017_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[1, 5]`.
	fn emit_consensus_errors(b: u32, ) -> Weight {
		Weight::from_parts(17_280_993_u64, 0)
			// Standard Error: 11_424
			.saturating_add(Weight::from_parts(3_172_369_u64, 0).saturating_mul(b as u64))
			.saturating_add(T::DbWeight::get().reads(2_u64))
	}
	// Storage: `DdcVerification::EraValidations` (r:1 w:1)
	// Proof: `DdcVerification::EraValidations` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_era_validations() -> Weight {
		Weight::from_parts(23_815_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn skip_dac_validation_to_era() -> Weight {
		Weight::from_parts(0_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::EraValidations` (r:1 w:1)
	// Proof: `DdcVerification::EraValidations` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[1, 5]`.
	fn set_prepare_era_for_payout(b: u32, ) -> Weight {
		Weight::from_parts(33_337_691_u64, 0)
			// Standard Error: 12_689
			.saturating_add(Weight::from_parts(380_174_u64, 0).saturating_mul(b as u64))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: `Staking::Ledger` (r:1 w:0)
	// Proof: `Staking::Ledger` (`max_values`: None, `max_size`: Some(1091), added: 3566, mode: `MaxEncodedLen`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorToStashKey` (r:0 w:1)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_validator_key() -> Weight {
		Weight::from_parts(29_155_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcClusters::Clusters` (r:1 w:0)
	// Proof: `DdcClusters::Clusters` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::BillingFingerprints` (r:1 w:1)
	// Proof: `DdcPayouts::BillingFingerprints` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn commit_billing_fingerprint() -> Weight {
		Weight::from_parts(40_165_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::BillingFingerprints` (r:1 w:0)
	// Proof: `DdcPayouts::BillingFingerprints` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::EraValidations` (r:1 w:1)
	// Proof: `DdcVerification::EraValidations` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn begin_billing_report() -> Weight {
		Weight::from_parts(45_956_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn begin_charging_customers() -> Weight {
		Weight::from_parts(32_561_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::BillingFingerprints` (r:1 w:0)
	// Proof: `DdcPayouts::BillingFingerprints` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcClusters::ClustersGovParams` (r:1 w:0)
	// Proof: `DdcClusters::ClustersGovParams` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcCustomers::Buckets` (r:500 w:500)
	// Proof: `DdcCustomers::Buckets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcCustomers::Ledger` (r:500 w:500)
	// Proof: `DdcCustomers::Ledger` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `System::Account` (r:2 w:2)
	// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	// Storage: `DdcPayouts::DebtorCustomers` (r:489 w:489)
	// Proof: `DdcPayouts::DebtorCustomers` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[1, 500]`.
	fn send_charging_customers_batch(b: u32, ) -> Weight {
		Weight::from_parts(149_151_000_u64, 0)
			// Standard Error: 236_904
			.saturating_add(Weight::from_parts(80_520_850_u64, 0).saturating_mul(b as u64))
			.saturating_add(RocksDbWeight::get().reads(10_u64))
			.saturating_add(RocksDbWeight::get().reads((3_u64).saturating_mul(b as u64)))
			.saturating_add(RocksDbWeight::get().writes(6_u64))
			.saturating_add(RocksDbWeight::get().writes((3_u64).saturating_mul(b as u64)))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcClusters::ClustersGovParams` (r:1 w:0)
	// Proof: `DdcClusters::ClustersGovParams` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `System::Account` (r:3 w:3)
	// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	// Storage: `DdcClusters::Clusters` (r:1 w:0)
	// Proof: `DdcClusters::Clusters` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `Staking::Validators` (r:2 w:0)
	// Proof: `Staking::Validators` (`max_values`: None, `max_size`: Some(45), added: 2520, mode: `MaxEncodedLen`)
	// Storage: `Staking::Bonded` (r:1 w:0)
	// Proof: `Staking::Bonded` (`max_values`: None, `max_size`: Some(72), added: 2547, mode: `MaxEncodedLen`)
	// Storage: `Staking::Ledger` (r:1 w:0)
	// Proof: `Staking::Ledger` (`max_values`: None, `max_size`: Some(1091), added: 3566, mode: `MaxEncodedLen`)
	// Storage: `Staking::Nominators` (r:1 w:0)
	// Proof: `Staking::Nominators` (`max_values`: None, `max_size`: Some(558), added: 3033, mode: `MaxEncodedLen`)
	fn end_charging_customers() -> Weight {
		Weight::from_parts(250_869_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(13_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn begin_rewarding_providers() -> Weight {
		Weight::from_parts(33_362_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::BillingFingerprints` (r:1 w:0)
	// Proof: `DdcPayouts::BillingFingerprints` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcNodes::StorageNodes` (r:500 w:500)
	// Proof: `DdcNodes::StorageNodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `System::Account` (r:501 w:501)
	// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[1, 500]`.
	fn send_rewarding_providers_batch(b: u32, ) -> Weight {
		Weight::from_parts(122_499_000_u64, 0)
			// Standard Error: 54_711
			.saturating_add(Weight::from_parts(76_199_725_u64, 0).saturating_mul(b as u64))
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(b as u64)))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
			.saturating_add(RocksDbWeight::get().writes((2_u64).saturating_mul(b as u64)))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn end_rewarding_providers() -> Weight {
		Weight::from_parts(34_094_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	// Storage: `DdcPayouts::ActiveBillingReports` (r:1 w:1)
	// Proof: `DdcPayouts::ActiveBillingReports` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::EraValidations` (r:1 w:1)
	// Proof: `DdcVerification::EraValidations` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcClusters::Clusters` (r:1 w:1)
	// Proof: `DdcClusters::Clusters` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn end_billing_report() -> Weight {
		Weight::from_parts(62_017_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	// Storage: `DdcVerification::ValidatorToStashKey` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorToStashKey` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[1, 5]`.
	fn emit_consensus_errors(b: u32, ) -> Weight {
		Weight::from_parts(17_280_993_u64, 0)
			// Standard Error: 11_424
			.saturating_add(Weight::from_parts(3_172_369_u64, 0).saturating_mul(b as u64))
			.saturating_add(RocksDbWeight::get().reads(2_u64))
	}
	// Storage: `DdcVerification::EraValidations` (r:1 w:1)
	// Proof: `DdcVerification::EraValidations` (`max_values`: None, `max_size`: None, mode: `Measured`)
	// Storage: `DdcVerification::ValidatorSet` (r:1 w:0)
	// Proof: `DdcVerification::ValidatorSet` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_era_validations() -> Weight {
		Weight::from_parts(23_815_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	fn skip_dac_validation_to_era() -> Weight {
		Weight::from_parts(0_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
}

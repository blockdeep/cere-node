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
// limitations under the License.

//! Autogenerated weights for pallet_recovery
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-06, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bm2`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/substrate
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_recovery
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./frame/recovery/src/weights.rs
// --header=./HEADER-APACHE2
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weights for pallet_recovery using the Substrate node and recommended hardware.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_recovery::WeightInfo for WeightInfo<T> {
	/// Storage: Recovery Proxy (r:1 w:0)
	/// Proof: Recovery Proxy (max_values: None, max_size: Some(80), added: 2555, mode: MaxEncodedLen)
	fn as_recovered() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `281`
		//  Estimated: `3545`
		// Minimum execution time: 10_405_000 picoseconds.
		Weight::from_parts(10_807_000, 3545)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: Recovery Proxy (r:0 w:1)
	/// Proof: Recovery Proxy (max_values: None, max_size: Some(80), added: 2555, mode: MaxEncodedLen)
	fn set_recovered() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 11_198_000 picoseconds.
		Weight::from_parts(11_459_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Recovery Recoverable (r:1 w:1)
	/// Proof: Recovery Recoverable (max_values: None, max_size: Some(351), added: 2826, mode: MaxEncodedLen)
	/// The range of component `n` is `[1, 9]`.
	fn create_recovery(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `175`
		//  Estimated: `3816`
		// Minimum execution time: 28_009_000 picoseconds.
		Weight::from_parts(28_755_652, 3816)
			// Standard Error: 3_536
			.saturating_add(Weight::from_parts(78_348, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Recovery Recoverable (r:1 w:0)
	/// Proof: Recovery Recoverable (max_values: None, max_size: Some(351), added: 2826, mode: MaxEncodedLen)
	/// Storage: Recovery ActiveRecoveries (r:1 w:1)
	/// Proof: Recovery ActiveRecoveries (max_values: None, max_size: Some(389), added: 2864, mode: MaxEncodedLen)
	fn initiate_recovery() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `272`
		//  Estimated: `3854`
		// Minimum execution time: 31_233_000 picoseconds.
		Weight::from_parts(31_508_000, 3854)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Recovery Recoverable (r:1 w:0)
	/// Proof: Recovery Recoverable (max_values: None, max_size: Some(351), added: 2826, mode: MaxEncodedLen)
	/// Storage: Recovery ActiveRecoveries (r:1 w:1)
	/// Proof: Recovery ActiveRecoveries (max_values: None, max_size: Some(389), added: 2864, mode: MaxEncodedLen)
	/// The range of component `n` is `[1, 9]`.
	fn vouch_recovery(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `360 + n * (64 ±0)`
		//  Estimated: `3854`
		// Minimum execution time: 20_542_000 picoseconds.
		Weight::from_parts(21_224_065, 3854)
			// Standard Error: 3_018
			.saturating_add(Weight::from_parts(171_994, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Recovery Recoverable (r:1 w:0)
	/// Proof: Recovery Recoverable (max_values: None, max_size: Some(351), added: 2826, mode: MaxEncodedLen)
	/// Storage: Recovery ActiveRecoveries (r:1 w:0)
	/// Proof: Recovery ActiveRecoveries (max_values: None, max_size: Some(389), added: 2864, mode: MaxEncodedLen)
	/// Storage: Recovery Proxy (r:1 w:1)
	/// Proof: Recovery Proxy (max_values: None, max_size: Some(80), added: 2555, mode: MaxEncodedLen)
	/// The range of component `n` is `[1, 9]`.
	fn claim_recovery(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `392 + n * (64 ±0)`
		//  Estimated: `3854`
		// Minimum execution time: 25_141_000 picoseconds.
		Weight::from_parts(25_880_238, 3854)
			// Standard Error: 3_156
			.saturating_add(Weight::from_parts(54_405, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Recovery ActiveRecoveries (r:1 w:1)
	/// Proof: Recovery ActiveRecoveries (max_values: None, max_size: Some(389), added: 2864, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `n` is `[1, 9]`.
	fn close_recovery(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `513 + n * (32 ±0)`
		//  Estimated: `3854`
		// Minimum execution time: 35_314_000 picoseconds.
		Weight::from_parts(36_380_338, 3854)
			// Standard Error: 7_396
			.saturating_add(Weight::from_parts(3_861, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: Recovery ActiveRecoveries (r:1 w:0)
	/// Proof: Recovery ActiveRecoveries (max_values: None, max_size: Some(389), added: 2864, mode: MaxEncodedLen)
	/// Storage: Recovery Recoverable (r:1 w:1)
	/// Proof: Recovery Recoverable (max_values: None, max_size: Some(351), added: 2826, mode: MaxEncodedLen)
	/// The range of component `n` is `[1, 9]`.
	fn remove_recovery(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `270 + n * (32 ±0)`
		//  Estimated: `3854`
		// Minimum execution time: 33_453_000 picoseconds.
		Weight::from_parts(34_078_626, 3854)
			// Standard Error: 2_563
			.saturating_add(Weight::from_parts(78_179, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Recovery Proxy (r:1 w:1)
	/// Proof: Recovery Proxy (max_values: None, max_size: Some(80), added: 2555, mode: MaxEncodedLen)
	fn cancel_recovered() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `281`
		//  Estimated: `3545`
		// Minimum execution time: 12_196_000 picoseconds.
		Weight::from_parts(12_580_000, 3545)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

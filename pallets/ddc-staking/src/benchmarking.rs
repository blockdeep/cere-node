//! DdcStaking pallet benchmarking.

use super::*;
use crate::Pallet as DdcStaking;
use ddc_primitives::{CDNNodePubKey, NodeType};
use testing_utils::*;

use frame_support::traits::{Currency, Get};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

pub use frame_benchmarking::{
	account, benchmarks, impl_benchmark_test_suite, whitelist_account, whitelisted_caller,
};
use frame_system::RawOrigin;

const USER_SEED: u32 = 999666;

benchmarks! {
	bond {
		let stash = create_funded_user::<T>("stash", USER_SEED, 100);
		let controller = create_funded_user::<T>("controller", USER_SEED, 100);
		let controller_lookup: <T::Lookup as StaticLookup>::Source
			= T::Lookup::unlookup(controller.clone());
		let node = NodePubKey::CDNPubKey(CDNNodePubKey::new([0; 32]));
		let amount = T::Currency::minimum_balance() * 10u32.into();
		whitelist_account!(stash);
	}: _(RawOrigin::Signed(stash.clone()), controller_lookup, node.clone(), amount)
	verify {
		assert!(Bonded::<T>::contains_key(stash));
		assert!(Ledger::<T>::contains_key(controller));
		assert!(Nodes::<T>::contains_key(node));
	}

	unbond {
		// clean up any existing state.
		clear_storages_and_cdns::<T>();

		let (stash, controller, _) = create_stash_controller_node::<T>(0, 100)?;
		let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created before")?;
		let original_bonded: BalanceOf<T> = ledger.active;
		let amount = T::Currency::minimum_balance() * 5u32.into(); // Half of total

		whitelist_account!(controller);
	}: _(RawOrigin::Signed(controller.clone()), amount)
	verify {
		let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created after")?;
		let new_bonded: BalanceOf<T> = ledger.active;
		assert!(original_bonded > new_bonded);
	}

	withdraw_unbonded {
		let (stash, controller, _) = create_stash_controller_node::<T>(0, 100)?;
		let amount = T::Currency::minimum_balance() * 5u32.into(); // Half of total
		DdcStaking::<T>::unbond(RawOrigin::Signed(controller.clone()).into(), amount)?;
		CurrentEra::<T>::put(EraIndex::max_value());
		let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created before")?;
		let original_total: BalanceOf<T> = ledger.total;
		whitelist_account!(controller);
	}: _(RawOrigin::Signed(controller.clone()))
	verify {
		let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created after")?;
		let new_total: BalanceOf<T> = ledger.total;
		assert!(original_total > new_total);
	}

	store {
		let (stash, controller, _) = create_stash_controller_node_with_balance::<T>(0, T::DefaultStorageBondSize::get())?;

		whitelist_account!(controller);
	}: _(RawOrigin::Signed(controller), ClusterId::from([1; 20]))
	verify {
		assert!(Storages::<T>::contains_key(&stash));
	}

	serve {
		let (stash, controller, _) = create_stash_controller_node_with_balance::<T>(0, T::DefaultCDNBondSize::get())?;

		whitelist_account!(controller);
	}: _(RawOrigin::Signed(controller), ClusterId::from([1; 20]))
	verify {
		assert!(CDNs::<T>::contains_key(&stash));
	}

	chill {
		// clean up any existing state.
		clear_storages_and_cdns::<T>();

		let (cdn_stash, cdn_controller, _) = create_stash_controller_node_with_balance::<T>(0, T::DefaultCDNBondSize::get())?;
		DdcStaking::<T>::serve(RawOrigin::Signed(cdn_controller.clone()).into(), ClusterId::from([1; 20]))?;
		assert!(CDNs::<T>::contains_key(&cdn_stash));
		CurrentEra::<T>::put(1);
		DdcStaking::<T>::chill(RawOrigin::Signed(cdn_controller.clone()).into())?;
		CurrentEra::<T>::put(1 + T::ClusterVisitor::get_chill_delay(ClusterId::from([1; 20]), NodeType::CDN));

		whitelist_account!(cdn_controller);
	}: _(RawOrigin::Signed(cdn_controller))
	verify {
		assert!(!CDNs::<T>::contains_key(&cdn_stash));
	}

	set_controller {
		let (stash, _, _) = create_stash_controller_node::<T>(USER_SEED, 100)?;
		let new_controller = create_funded_user::<T>("new_controller", USER_SEED, 100);
		let new_controller_lookup = T::Lookup::unlookup(new_controller.clone());
		whitelist_account!(stash);
	}: _(RawOrigin::Signed(stash), new_controller_lookup)
	verify {
		assert!(Ledger::<T>::contains_key(&new_controller));
	}

	set_node {
		let (stash, _, _) = create_stash_controller_node::<T>(USER_SEED, 100)?;
		let new_node = NodePubKey::CDNPubKey(CDNNodePubKey::new([1; 32]));
		whitelist_account!(stash);
	}: _(RawOrigin::Signed(stash), new_node.clone())
	verify {
		assert!(Nodes::<T>::contains_key(&new_node));
	}
}

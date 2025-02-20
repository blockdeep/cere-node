//! Test utilities

#![allow(dead_code)]

use ddc_primitives::traits::staking::{StakingVisitor, StakingVisitorError};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, ConstU64, Everything},
	weights::constants::RocksDbWeight,
};
use frame_system::mocking::{MockBlock, MockUncheckedExtrinsic};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature,
};

use crate::{self as pallet_ddc_nodes, *};

/// The AccountId alias in this test module.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub(crate) type AccountIndex = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;
pub type Signature = MultiSignature;

type UncheckedExtrinsic = MockUncheckedExtrinsic<Test>;
type Block = MockBlock<Test>;

construct_runtime!(
	pub struct Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		DdcNodes: pallet_ddc_nodes::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub static ExistentialDeposit: Balance = 1;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = RocksDbWeight;
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type Block = Block;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<1024>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type RuntimeFreezeReason = ();
	type MaxFreezes = ();
	type MaxHolds = ();
	type RuntimeHoldReason = ();
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<5>;
	type WeightInfo = ();
}

impl crate::pallet::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type StakingVisitor = TestStakingVisitor;
	type WeightInfo = ();
}

pub struct TestStakingVisitor;
impl<T: Config> StakingVisitor<T> for TestStakingVisitor {
	fn has_activated_stake(
		_node_pub_key: &NodePubKey,
		_cluster_id: &ClusterId,
	) -> Result<bool, StakingVisitorError> {
		Ok(false)
	}
	fn has_stake(_node_pub_key: &NodePubKey) -> bool {
		false
	}
	fn has_chilling_attempt(_node_pub_key: &NodePubKey) -> Result<bool, StakingVisitorError> {
		Ok(false)
	}

	fn stash_by_ctrl(_controller: &T::AccountId) -> Result<T::AccountId, StakingVisitorError> {
		todo!()
	}
}

pub(crate) type TestRuntimeCall = <Test as frame_system::Config>::RuntimeCall;

pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build(self) -> TestExternalities {
		sp_tracing::try_init_simple();

		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
		let account_id1 = AccountId::from([1; 32]);
		let account_id2 = AccountId::from([2; 32]);
		let _ = pallet_balances::GenesisConfig::<Test> {
			balances: vec![(account_id1, 100), (account_id2, 100)],
		}
		.assimilate_storage(&mut t);

		TestExternalities::new(t)
	}
	pub fn build_and_execute(self, test: impl FnOnce()) {
		sp_tracing::try_init_simple();
		let mut ext = self.build();
		ext.execute_with(test);
	}
}

use frame_support::{
	assert_ok, ord_parameter_types, parameter_types, traits::Everything, weights::Weight,
};
use frame_system::{self as system};
pub use pallet_balances as balances;
use sp_core::H256;
use sp_runtime::{
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
	BuildStorage, Perbill,
};

use crate::{self as bridge, *};

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_parts(1024, 0);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const MaxLocks: u32 = 50;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Block = Block;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	// type ModuleToIndex = ();
	type PalletInfo = PalletInfo;
	// type MaxLocks = MaxLocks;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

ord_parameter_types! {
	pub const One: u64 = 1;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type FreezeIdentifier = ();
	type RuntimeFreezeReason = ();
	type MaxFreezes = ();
	type MaxHolds = ();
	type RuntimeHoldReason = ();
}

parameter_types! {
	pub const TestChainId: u8 = 5;
	pub const ProposalLifetime: u64 = 50;
	pub BridgeAccountId: u64 = AccountIdConversion::<u64>::into_account_truncating(&MODULE_ID);
}

impl crate::pallet::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type Proposal = RuntimeCall;
	type ChainIdentity = TestChainId;
	type ProposalLifetime = ProposalLifetime;
	type BridgeAccountId = BridgeAccountId;
	type WeightInfo = ();
}

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub struct Test
	{
		System: system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Bridge: bridge::{Pallet, Call, Storage, Event<T>},
	}
);

// pub const BRIDGE_ID: u64 =
pub const RELAYER_A: u64 = 0x2;
pub const RELAYER_B: u64 = 0x3;
pub const RELAYER_C: u64 = 0x4;
pub const ENDOWED_BALANCE: u64 = 100_000_000;
pub const TEST_THRESHOLD: u32 = 2;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let bridge_id = AccountIdConversion::into_account_truncating(&MODULE_ID);
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances: vec![(bridge_id, ENDOWED_BALANCE)] }
		.assimilate_storage(&mut t)
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn new_test_ext_initialized(
	src_id: ChainId,
	r_id: ResourceId,
	resource: Vec<u8>,
) -> sp_io::TestExternalities {
	let mut t = new_test_ext();
	t.execute_with(|| {
		// Set and check threshold
		assert_ok!(Bridge::set_threshold(RuntimeOrigin::root(), TEST_THRESHOLD));
		assert_eq!(Bridge::relayer_threshold(), TEST_THRESHOLD);
		// Add relayers
		assert_ok!(Bridge::add_relayer(RuntimeOrigin::root(), RELAYER_A));
		assert_ok!(Bridge::add_relayer(RuntimeOrigin::root(), RELAYER_B));
		assert_ok!(Bridge::add_relayer(RuntimeOrigin::root(), RELAYER_C));
		// Whitelist chain
		assert_ok!(Bridge::whitelist_chain(RuntimeOrigin::root(), src_id));
		// Set and check resource ID mapped to some junk data
		assert_ok!(Bridge::set_resource(RuntimeOrigin::root(), r_id, resource));
		assert_eq!(Bridge::resource_exists(r_id), true);
	});
	t
}

// Checks events against the latest. A contiguous set of events must be provided. They must
// include the most recent event, but do not have to include every past event.
pub fn assert_events(mut expected: Vec<RuntimeEvent>) {
	let mut actual: Vec<RuntimeEvent> =
		system::Pallet::<Test>::events().iter().map(|e| e.event.clone()).collect();

	expected.reverse();

	for evt in expected {
		let next = actual.pop().expect("event expected");
		assert_eq!(next, evt, "Events don't match (actual,expected)");
	}
}

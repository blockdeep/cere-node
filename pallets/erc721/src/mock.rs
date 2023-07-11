#![cfg(test)]

use frame_support::{ord_parameter_types, parameter_types, traits::Everything, weights::Weight};
use frame_system::{self as system};
use sp_core::{hashing::blake2_128, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, Block as BlockT, IdentityLookup},
	BuildStorage, Perbill,
};

use crate::{self as erc721, Config};
pub use pallet_balances as balances;
use pallet_chainbridge as bridge;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_ref_time(1024);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = balances::AccountData<u64>;
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
}

parameter_types! {
	pub Erc721Id: bridge::ResourceId = bridge::derive_resource_id(1, &blake2_128(b"NFT"));
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Identifier = Erc721Id;
}

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, u64, RuntimeCall, ()>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: system::{Pallet, Call, Event<T>},
		Balances: balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Erc721: erc721::{Pallet, Call, Storage, Event<T>},
	}
);

pub const USER_A: u64 = 0x1;
pub const USER_B: u64 = 0x2;
pub const USER_C: u64 = 0x3;
pub const ENDOWED_BALANCE: u64 = 100_000_000;

pub fn new_test_ext() -> sp_io::TestExternalities {
	GenesisConfig {
		balances: balances::GenesisConfig { balances: vec![(USER_A, ENDOWED_BALANCE)] },
	}
	.build_storage()
	.unwrap()
	.into()
}

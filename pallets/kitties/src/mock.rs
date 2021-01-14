//use super::*;
//use sp_core::H256;
//use frame_support::{impl_outer_event,impl_outer_origin,parameter_types,weights::Weight,traits::{OnFinalize,OnInitialize}};
//use sp_runtime::{traits::{BlakeTwo256,IdentityLookup}, testing::Header, Perbill,};
//use frame_system as system;
//use frame_system::Origin;

use crate::*;
use balances;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};
use frame_system as system;
use frame_support::{ impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
use sp_io;



impl_outer_origin! {
	pub enum origin for TestKitty {}
}


#[derive(Clone, Eq, PartialEq,Debug)]
pub struct TestKitty;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	pub const ExistentialDeposit: u64 = 1;
}
mod kitties {
	pub use crate::Event;
}


impl system::Trait for TestKitty {
	type BaseCallFilter = ();
	type Origin = origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl TraitTest for TestKitty {
	type Event = ();
	// type Event = TestEvent;
	type Randomness = Randomness;
	type KittyIndex = u32;
	//type Currency = balances::Module<Self>;
}

pub type Kitties = Module<TestKitty>;

pub type System = frame_system::Module<TestKitty>;

pub fn new_test_ext() -> sp_io::TestExternalities{

	system::GenesisConfig::default().build_storage::<TestKitty>().unwrap().into()
}

type Randomness = pallet_randomness_collective_flip::Module<TestKitty>;


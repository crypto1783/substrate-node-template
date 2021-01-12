//编译条件 只有满足条件时才会编译
#![cfg_attr(not(feature = "std"), no_std)]
#[warn(unused_imports)]
//#![no_std]

use codec::{Encode,Decode};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure,StorageValue, dispatch, StorageMap,traits::Randomness
};
use sp_io::hashing::blake2_128;
use sp_runtime::DispatchError;
use frame_system::ensure_signed;
use sp_std::vec::Vec;
use pallet_balances as balances;
//use pallet_randomness_collective_flip;
//use parity_scale_codec::Encode;
//use sp_core::blake2_128;

//id
type KittyIndex = u32;

//DNA u8类型 长度为16的数组
#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

// 2. Configuration
/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait TraitTest: frame_system::Trait {
	// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Randomness: Randomness<Self::Hash>;
}

// 3. Storage
// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {

	//这个写法上是substrte特有的，不是rust语法，所以参照写即可
    trait Store for Module<T: TraitTest> as TemplateModule {
    	pub Kitties get(fn kitties):map hasher(blake2_128_concat) KittyIndex => Option<Kitty>;

    	pub KittiesCount get(fn kitties_count):KittyIndex;
    	pub KittyOwners get(fn kitty_owner):map hasher(blake2_128_concat) KittyIndex => Option<T::AccountId>;

    }
}

// 4. Events
// Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event! {
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {

    	Created(AccountId, KittyIndex),

    	Transferred(AccountId, AccountId,KittyIndex),

    }
}

// 5. Errors
// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: TraitTest> {
		KittiesCountOverflow,
		InvalidKittyId,
		RequireDifferentParent,
    }
}

// 6. Callable Functions
// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {

    pub struct Module<T: TraitTest> for enum Call where origin: T::Origin {

    type Error = Error<T>;
	fn deposit_event() = default;
    	#[weight = 0]
		pub fn create(orign){
			let sender = ensure_signed(orign)?;
			let kitty_id = Self::next_kitty_id()?;
			let dna = Self::random_value(&sender);
			let kitty = Kitty(dna);
			Self::insert_kitty(&sender, kitty_id, kitty);
			Self::deposit_event(RawEvent::Created(sender,kitty_id));

		}

		#[weight = 0]
		pub fn transfer(orign, to:T::AccountId, kitty_id:KittyIndex)
		{
			let sender = ensure_signed(orign)?;
			<KittyOwners<T>>::insert(kitty_id,to.clone());
			Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
		}

		#[weight = 0]
		pub fn breed(orign, kitty_id_1:KittyIndex, kitty_id_2:KittyIndex)
		{
			let sender = ensure_signed(orign)?;
			let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
			Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
		}

    }


}

impl<T: TraitTest> Module<T>{

	fn next_kitty_id() -> sp_std::result::Result<KittyIndex,DispatchError>{
		let kitty_id = Self::kitties_count();
		if kitty_id == KittyIndex::max_value()
		{
			return Err(Error::<T>::KittiesCountOverflow.into());
		}
		Ok(kitty_id)
	}

	fn random_value(sender:&T::AccountId) -> [u8;16]{

		let paylaod = (
			T::Randomness::random_seed(),
			&sender,
			<frame_system::Module<T>>::extrinsic_index(),
			);
		paylaod.using_encoded(blake2_128)

	}

	fn insert_kitty(owner:&T::AccountId,kitty_id:KittyIndex, kitty:Kitty)
	{
		Kitties::insert(kitty_id, kitty);
		KittiesCount::put(kitty_id + 1);
		<KittyOwners<T>>::insert(kitty_id, owner);
	}
	fn combine_dna(dna1:u8,dna2:u8,selector:u8) -> u8 {
		(selector & dna1) | (!selector & dna2)
	}

	fn do_breed(sender:&T::AccountId,kitty_id_1:KittyIndex,kitty_id_2:KittyIndex) -> sp_std::result::Result<KittyIndex,DispatchError>
	{
		let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
		let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;
		ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

		let kitty_id = Self::next_kitty_id()?;
		let kitty1_dna = kitty1.0;
		let kitty2_dna =  kitty2.0;
		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8;16];

		for i in 0..kitty1_dna.len(){

			new_dna[i] = Self::combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}

		Self::insert_kitty(sender,kitty_id,Kitty(new_dna));
		Ok(kitty_id)

	}
}

#[cfg(test)]
mod tests{
	use super::*;
	use sp_core::H256;
	use frame_support::{impl_outer_event,impl_outer_origin,parameter_types,weights::Weight,traits::{OnFinalize,OnInitialize}};
	use sp_runtime::{traits::{BlakeTwo256,IdentityLookup}, testing::Header, Perbill,};
	use frame_system as system;
	use frame_system::Origin;
	impl_outer_origin! {
	pub enum origin for Test {}
}


	#[derive(Clone, Eq, PartialEq,Debug)]
	pub struct Test;
	parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	pub const ExistentialDeposit: u64 = 1;
}

	impl system::Trait for Test {
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

	type Randomness = pallet_randomness_collective_flip::Module<Test>;

	impl TraitTest for Test {
		type Event = ();
		// type Event = TestEvent;
		type Randomness = Randomness;
		//type KittyIndex = u32;
		//type Currency = balances::Module<Self>;
	}

	pub type Kitties = Module<Test>;
	fn new_test_ext() -> sp_io::TestExternalities{

		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn owned_kitties_can_append_values(){
		new_test_ext().execute_with(||{
			assert_eq!(Kitties::create(origin::signed(1),),Ok(()));
		})
	}


}
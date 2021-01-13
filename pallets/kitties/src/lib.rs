//编译条件 只有满足条件时才会编译
#![cfg_attr(not(feature = "std"), no_std)]
#[warn(unused_imports)]
//#![no_std]

use codec::{Encode,Decode};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure,StorageValue, dispatch, StorageMap,traits::Randomness,Parameter
};
use sp_io::hashing::blake2_128;

use frame_system::ensure_signed;
use sp_std::vec::Vec;
use pallet_balances as balances;
use sp_runtime::{DispatchError,traits::{AtLeast32Bit,Bounded}};
//use pallet_randomness_collective_flip;
//use parity_scale_codec::Encode;
//use sp_core::blake2_128;

//id
//type KittyIndex = u32;

//DNA u8类型 长度为16的数组
#[derive(Encode, Decode)]
pub struct Kitty(
	pub [u8; 16]
);

// 2. Configuration
/// Configure the pallet by specifying the parameters and types on which it depends.
/// 定义一个trait,trait的名字叫做Trait,这个trait继承自frame_system::Trait
pub trait TraitTest: frame_system::Trait {
	// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Randomness: Randomness<Self::Hash>;

	// 定义 KittyIndex 类型，要求实现指定的 trait
	// Parameter 表示可以用于函数参数传递
	// AtLeast32Bit 表示转换为 u32 不会造成数据丢失
	// Bounded 表示包含上界和下界
	// Default 表示有默认值
	// Copy 表示可以实现 Copy这个trait
	// 类型KiityIndex需要同时实现以下5种trait
	type KittyIndex: Parameter + AtLeast32Bit + Bounded + Default + Copy;
}

// 3. Storage
// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {

	//这个写法上是substrte特有的，不是rust语法，所以参照写即可
    trait Store for Module<T: TraitTest> as TemplateModule {

        /// key是kitty每次新增的序列id, vulue 是DNA
    	pub Kitties get(fn kitties):map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;

		/// KittiesCount这个存储单元用于存放一个计数器的值,每新增一个kitty计数器+1
    	pub KittiesCount get(fn kitties_count):T::KittyIndex;

		/// key是序列，value是所有者的账号id， Option<T::AccountId>语法含义是什么
    	pub KittyOwners get(fn kitty_owner):map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;

    }
}

// 4. Events
// Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event! {

    /// 这里的KittyIndex = <T as Trait>::KittyIndex的语法是什么写法？
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId,KittyIndex = <T as TraitTest>::KittyIndex {
		//
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

	/// 定义了一个结构体,名字为Module,这是一个带有泛型的结构体,泛型的要求是必须实现了TraitTest这个Trait
	///
    pub struct Module<T: TraitTest> for enum Call where origin: T::Origin {

		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 0]
		pub fn create(orign){

			let sender = ensure_signed(orign)?;

			/// Self是指调用方这个对象，这里因为当前这个方法create所在的struct同时是next_kitty_id所在impl实现的结构体Module
			let kitty_id = Self::next_kitty_id()?;

			/// 获取一个DNA数组
			let dna = Self::random_value(&sender);

			/// 实例化结构体Kitty
			let kitty = Kitty(dna);

			///
			Self::insert_kitty(&sender, kitty_id, kitty);

			Self::deposit_event(RawEvent::Created(sender,kitty_id));

		}

		#[weight = 0]
		pub fn transfer(orign, to:T::AccountId, kitty_id:T::KittyIndex)
		{
			let sender = ensure_signed(orign)?;
			<KittyOwners<T>>::insert(kitty_id,to.clone());
			Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
		}

		#[weight = 0]
		pub fn breed(orign, kitty_id_1:T::KittyIndex, kitty_id_2:T::KittyIndex)
		{
			let sender = ensure_signed(orign)?;
			let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
			Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
		}

}


}

/// 这里的Module是应该是90行中decl_module中定义的的结构体module,这里是对结构体Module的方法实现，rust中结构体可以有多个impl
/// 结构体的方法实现并不需要像trait那样先进行方法定义
/// 这个结构体 有一个泛型T，这个范型要求实现TraitTest这个trait,如果这个结构体的实例化对象没有实现TraitTest,则当前这个impl对这个结构体的实现无效,这就是有条件地实现结构体的方法
impl<T: TraitTest> Module<T>{

	///T::KittyIndex是指实现TraitTest这个trait的对象中的成员变量KittyIndex
	fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex,DispatchError>{

		/// Self代表的时候这个方法next_kitty_id()的调用方，KittiesCount这个存储单元在使用时有一个可选的方法kitties_count()
		let kitty_id = Self::kitties_count();

		/// 计数器达到了i32的最大值则抛出异常
		if kitty_id == T::KittyIndex::max_value()
		{
			return Err(Error::<T>::KittiesCountOverflow.into());
		}

		Ok(kitty_id)
	}

	///生成一个u8类型长度为16的数组，算是一个伪随机的数
	fn random_value(sender:&T::AccountId) -> [u8;16]{

		let paylaod = (
			T::Randomness::random_seed(),
			&sender,
			<frame_system::Module<T>>::extrinsic_index(),
			);

		paylaod.using_encoded(blake2_128)
	}

	/// 插入kitty的值到存储单元
	/// @owner：调用者sender
	/// @kitty_id：是一个每次新增kitty都+1的计数器
	/// @kitty: kitty数据是一个包含一个数组的结构体，数据内容是伪随机数据DNA
	fn insert_kitty(owner:&T::AccountId, kitty_id:T::KittyIndex, kitty:Kitty)
	{
		//
		// 这里的存储单元为什么要这么写? into是什么方法
		<KittiesCount::<T>>::put(kitty_id+1.into());
		<KittyOwners::<T>>::insert(kitty_id, owner);
		<Kitties::<T>>::insert(kitty_id, kitty);
		 //Kitties::insert(kitty_id, kitty);
		//KittiesCount::put(kitty_id + 1.into());
		//<KittyOwners<T>>::insert(kitty_id, owner);
	}


	fn combine_dna(dna1:u8,dna2:u8,selector:u8) -> u8 {
		(selector & dna1) | (!selector & dna2)
	}

	fn do_breed(sender:&T::AccountId,kitty_id_1:T::KittyIndex,kitty_id_2:T::KittyIndex) -> sp_std::result::Result<T::KittyIndex,DispatchError>
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
		type KittyIndex = u32;
		//type Currency = balances::Module<Self>;
	}

	pub type Kitties = Module<Test>;

	pub type System = frame_system::Module<Test>;
	fn run_to_block( n: u64) {
		while System::block_number() < n {
			Kitties::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
			System::set_block_number(System::block_number()+1);
			System::on_initialize(System::block_number());
			Kitties::on_initialize(System::block_number());
		}
	}


	fn new_test_ext() -> sp_io::TestExternalities{

		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn owned_kitties_can_append_values(){
		new_test_ext().execute_with(||{
			run_to_block(10);
			assert_eq!(Kitties::create(origin::signed(1),),Ok(()));
		})
	}


}
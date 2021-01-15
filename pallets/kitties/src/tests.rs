use crate::{Error, mock::*, Event};
use frame_support::{assert_ok, assert_noop};
use frame_support::traits::{OnFinalize, OnInitialize};
use frame_system::{EventRecord, Phase};
use pallet_balances::RawEvent;

fn run_to_block( n: u64) {
	while System::block_number() < n {
		Kitties::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number()+1);
		System::on_initialize(System::block_number());
	}
}
//
#[test]
fn owned_kitties_can_append_values(){
	new_test_ext().execute_with(||{
		run_to_block(10);
		assert_eq!(Kitties::create(origin::signed(1),),Ok(()));
		Kitties::create(origin::signed(1));

		assert_eq!(System::events()[0].event, TestEvent::balances(RawEvent::Reserved(1, 5000)));
		assert_eq!(System::events()[1].event, TestEvent::kitties( Event::<TestKitty>::Created( 1u64 , 0) ));
		Kitties::on_initialize(System::block_number());
	})
}
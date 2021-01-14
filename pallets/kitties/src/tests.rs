use crate::{Error, mock::*, Event};
use frame_support::{assert_ok, assert_noop};
use frame_support::traits::{OnFinalize, OnInitialize};
use frame_system::{EventRecord, Phase};

fn run_to_block( n: u64) {
	while System::block_number() < n {
		Kitties::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number()+1);
		System::on_initialize(System::block_number());
	}
}

/*
impl_outer_event! {
	pub enum TestEvent for TestRuntime {
		//simple_event,
		//frame_system<T>,

	}
}
*/
#[test]
fn owned_kitties_can_append_values(){
	new_test_ext().execute_with(||{
		run_to_block(10);
		assert_eq!(Kitties::create(origin::signed(1),),Ok(()));
		Kitties::create(origin::signed(1));
		let events = System::events();
		/*
		assert_eq!(
			System::events(),
			vec![EventRecord {
				phase: Phase::Initialization,
				event: TestEvent::simple_event(Event::EmitInput(32)),
				topics: vec![],
			}]
		);*/

		Kitties::on_initialize(System::block_number());
	})
}
use crate as pallet_template;
use frame_support::traits::{ConstU16, ConstU64};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use frame_support::parameter_types;
use frame_support_test::TestRandomness;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>}, // new code
		TemplateModule: pallet_template::{Pallet, Call, Storage, Event<T>},
		SchellingGameShared: schelling_game_shared::{Pallet, Call, Storage, Event<T>},
		ProfileValidation: profile_validation::{Pallet, Call, Storage, Event<T>},
		SortitionSumGame: sortition_sum_game::{Pallet, Call, Storage, Event<T>},
	
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type AccountData = pallet_balances::AccountData<u64>; // New code

}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl schelling_game_shared::Config for Test {
	type Event = Event;
	type Currency = Balances; // New code
	type RandomnessSource = TestRandomness<Self>;
	type Slash = ();
	type Reward = ();
	type SortitionSumGameSource = SortitionSumGame;
}

impl sortition_sum_game::Config for Test {
	type Event = Event;
}

impl profile_validation::Config for Test {
	type Event = Event;
	type Currency = Balances; // New code
	type SchellingGameSharedSource = SchellingGameShared;
}
impl pallet_template::Config for Test {
	type Event = Event;
	type ProfileValidationSource = ProfileValidation;
	type Currency = Balances; // New code
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

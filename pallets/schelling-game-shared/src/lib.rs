#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod extras;
mod types;

use crate::types::{CommitVote, DrawJurorsLimit, Period, SchellingGameType, StakingTime, VoteStatus};
use frame_support::pallet_prelude::*;
use frame_support::sp_runtime::traits::{CheckedAdd, CheckedMul, CheckedSub};
use frame_support::sp_runtime::SaturatedConversion;
use frame_support::sp_std::prelude::*;
use frame_support::{
	traits::{
		Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced, ReservableCurrency,
		WithdrawReasons,
	},
	PalletId,
};
use frame_system::pallet_prelude::*;
use num_integer::Roots;
use sortition_sum_game::types::SumTreeName;
use sortition_sum_game_link::SortitionSumGameLink;
use frame_support::{traits::Randomness};

pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::PositiveImbalance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type SortitionSumGameSource: SortitionSumGameLink<SumTreeName = SumTreeName, AccountId=Self::AccountId>;

	
		type Currency: ReservableCurrency<Self::AccountId>;

		type RandomnessSource: Randomness<Self::Hash, Self::BlockNumber>;

		/// Handler for the unbalanced increment when rewarding (minting rewards)
		type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;

		/// Handler for the unbalanced decrement when slashing (burning collateral)
		type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Schelling Game Storage:

	#[pallet::storage]
	pub type Nonce<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_period)]
	pub type PeriodName<T> = StorageMap<_, Blake2_128Concat, SumTreeName, Period>;

	#[pallet::type_value]
	pub fn DefaultMinBlockTime<T: Config>() -> StakingTime<BlockNumberOf<T>> {
		let staking_time = StakingTime {
			min_short_block_length: 50u128.saturated_into::<BlockNumberOf<T>>(),
			min_long_block_length: 50u128.saturated_into::<BlockNumberOf<T>>(),
		};
		staking_time
		// 3 days (43200), 10 days (144000)
		// 15 mins (150)
		// 5 mins (50)
	}

	#[pallet::storage]
	#[pallet::getter(fn min_block_time)]
	pub type MinBlockTime<T> = StorageMap<
		_,
		Blake2_128Concat,
		SchellingGameType,
		StakingTime<BlockNumberOf<T>>,
		ValueQuery,
		DefaultMinBlockTime<T>,
	>;

	#[pallet::type_value]
	pub fn DefaultMinStake<T: Config>() -> BalanceOf<T> {
		100u128.saturated_into::<BalanceOf<T>>()
	}

	#[pallet::storage]
	#[pallet::getter(fn min_juror_stake)]
	pub type MinJurorStake<T> = StorageMap<
		_,
		Blake2_128Concat,
		SchellingGameType,
		BalanceOf<T>,
		ValueQuery,
		DefaultMinStake<T>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn draws_in_round)]
	pub type DrawsInRound<T> = StorageMap<_, Blake2_128Concat, SumTreeName, u64, ValueQuery>; // A counter of draws made in the current round.

	#[pallet::storage]
	#[pallet::getter(fn staking_start_time)]
	pub type StakingStartTime<T> =
		StorageMap<_, Blake2_128Concat, SumTreeName, BlockNumberOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn commit_start_time)]
	pub type CommitStartTime<T> =
		StorageMap<_, Blake2_128Concat, SumTreeName, BlockNumberOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn vote_start_time)]
	pub type VoteStartTime<T> =
		StorageMap<_, Blake2_128Concat, SumTreeName, BlockNumberOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn  drawn_jurors)]
	pub type DrawnJurors<T: Config> =
		StorageMap<_, Blake2_128Concat, SumTreeName, Vec<T::AccountId>, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn unstaked_jurors)]
	pub type UnstakedJurors<T: Config> =
		StorageMap<_, Blake2_128Concat, SumTreeName, Vec<T::AccountId>, ValueQuery>;

	#[pallet::type_value]
	pub fn DefaultDrawJurorsLimitNum<T: Config>() -> DrawJurorsLimit {
		let draw_juror_limit = DrawJurorsLimit { max_draws: 5, max_draws_appeal: 10 };
		// change max draws more than 30 in production
		draw_juror_limit
	}

	#[pallet::storage]
	#[pallet::getter(fn draw_jurors_for_profile_limit)]
	pub type DrawJurorsLimitNum<T> = StorageMap<
		_,
		Blake2_128Concat,
		SchellingGameType,
		DrawJurorsLimit,
		ValueQuery,
		DefaultDrawJurorsLimitNum<T>,
	>;
	#[pallet::storage]
	#[pallet::getter(fn vote_commits)]
	pub type VoteCommits<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		SumTreeName,
		Blake2_128Concat,
		T::AccountId,
		CommitVote,
	>;

	#[pallet::storage]
	#[pallet::getter(fn decision_count)]
	pub type DecisionCount<T> =
		StorageMap<_, Blake2_128Concat, SumTreeName, (u64, u64), ValueQuery>;
	#[pallet::type_value]
	pub fn DefaultJurorIncentives<T: Config>() -> (u64, u64) {
		(100, 100)
	}

	#[pallet::storage]
	#[pallet::getter(fn juror_incentives)]
	pub type JurorIncentives<T> = StorageMap<
		_,
		Blake2_128Concat,
		SchellingGameType,
		(u64, u64), // (looser burn, winner mint)
		ValueQuery,
		DefaultJurorIncentives<T>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn juror_incentive_distribution)]
	pub type JurorsIncentiveDistribution<T: Config> =
		StorageMap<_, Blake2_128Concat, SchellingGameType, Vec<T::AccountId>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		PeriodExists,
		StakingPeriodNotOver,
		MaxJurorNotDrawn,
		CommitPeriodNotOver,
		VotePeriodNotOver,
		PeriodDoesNotExists,
		PeriodDontMatch,
		StakeLessThanMin,
		AlreadyStaked,
		MaxDrawExceeded, 
		SelectedAsJuror,
		AlreadyUnstaked,
		StakeDoesNotExists,
		JurorDoesNotExists,
		VoteStatusNotCommited,
		NotValidChoice,
		CommitDoesNotMatch,
		CommitDoesNotExists
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}

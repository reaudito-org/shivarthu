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

use frame_support::sp_runtime::traits::Saturating;
use frame_support::sp_runtime::SaturatedConversion;
use frame_support::sp_std::prelude::*;
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure, fail,
};
use frame_support::{
	traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons},
	PalletId,
};
use pallet_support::{
	ensure_content_is_valid, new_who_and_when, remove_from_vec, Content, PositiveExternalityPostId,
	WhoAndWhen, WhoAndWhenOf,
};
use schelling_game_shared::types::{Period, RangePoint, SchellingGameType};
use schelling_game_shared_link::SchellingGameSharedLink;
use shared_storage_link::SharedStorageLink;
use sortition_sum_game::types::SumTreeName;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type SumTreeNameType<T> = SumTreeName<AccountIdOf<T>, BlockNumberOf<T>>;
type DeparmentId = u128;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type SharedStorageSource: SharedStorageLink<AccountId = AccountIdOf<Self>>;
		type SchellingGameSharedSource: SchellingGameSharedLink<
			SumTreeName = SumTreeName<Self::AccountId, Self::BlockNumber>,
			SchellingGameType = SchellingGameType,
			BlockNumber = Self::BlockNumber,
			AccountId = AccountIdOf<Self>,
			Balance = BalanceOf<Self>,
			RangePoint = RangePoint,
			Period = Period,
		>;
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::type_value]
	pub fn MinimumDepartmentStake<T: Config>() -> BalanceOf<T> {
		10000u128.saturated_into::<BalanceOf<T>>()
	}

	#[pallet::storage]
	#[pallet::getter(fn department_stake)]
	pub type DepartmentStakeBalance<T: Config> =
		StorageMap<_, Twox64Concat, DeparmentId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn validation_department_block_number)]
	pub type ValidationDepartmentBlock<T: Config> =
		StorageMap<_, Blake2_128Concat, DeparmentId, BlockNumberOf<T>, ValueQuery>;

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
		LessThanMinStake,
		CannotStakeNow,
		ChoiceOutOfRange,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Should user need to stake every round?? or stake once and keep minimum stake balance. ✔️
		// User can on and off validation ✔️
		// Every 6 months validation (To do)
		// Start time-> First 10 days, any juror can stake, and change to stake period
		// Add the blocknumber when positive externality score is added as (u8, blocknumber) tuple.

		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn create_positive_externality_post(
		// 	origin: OriginFor<T>,
		// 	content: Content,
		// ) -> DispatchResult {
		// 	let creator = ensure_signed(origin)?;

		// 	ensure_content_is_valid(content.clone())?;
		// 	T::SharedStorageSource::check_citizen_is_approved_link(creator.clone())?;

		// 	let new_post_id = Self::next_positive_externality_post_id();

		// 	let new_post: PositiveExternalityPost<T> =
		// 		PositiveExternalityPost::new(new_post_id, creator.clone(), content.clone());

		// 	PositiveExternalityEvidence::<T>::mutate(creator, |ids| ids.push(new_post_id));

		// 	PositiveExternalityPostById::insert(new_post_id, new_post);
		// 	NextPositiveExternalityPostId::<T>::mutate(|n| {
		// 		*n += 1;
		// 	});

		// 	// emit event

		// 	Ok(())
		// }

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_department_stake(
			origin: OriginFor<T>,
			department_id: DeparmentId,
			deposit: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Check user has done kyc
			let _ = T::Currency::withdraw(
				&who,
				deposit,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?;
			let stake = DepartmentStakeBalance::<T>::get(department_id);
			let total_balance = stake.saturating_add(deposit);
			DepartmentStakeBalance::<T>::insert(department_id, total_balance);

			// emit event
			Ok(())
		}

		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn set_validate_positive_externality(
		// 	origin: OriginFor<T>,
		// 	value: bool,
		// ) -> DispatchResult {
		// 	let who = ensure_signed(origin)?;
		// 	// Check user has done kyc

		// 	ValidatePositiveExternality::<T>::insert(&who, value);
		// 	// emit event
		// 	Ok(())
		// }

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn apply_staking_period(
			origin: OriginFor<T>,
			department_id: DeparmentId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Self::ensure_validation_on_positive_externality(user_to_calculate.clone())?;
			Self::ensure_min_stake_deparment(department_id)?;

			let pe_block_number = <ValidationDepartmentBlock<T>>::get(department_id);
			let now = <frame_system::Pallet<T>>::block_number();
			let three_month_number = (3 * 30 * 24 * 60 * 60) / 6;
			let three_month_block = Self::u64_to_block_saturated(three_month_number);
			let modulus = now % three_month_block;
			let storage_main_block = now - modulus;
			// println!("{:?}", now);
			// println!("{:?}", three_month_number);
			// println!("{:?}", storage_main_block);
			// println!("{:?}", pe_block_number);

			let key = SumTreeName::DepartmentScore {
				department_id,
				block_number: storage_main_block.clone(),
			};

			// let game_type = SchellingGameType::PositiveExternality;

			if storage_main_block > pe_block_number {
				<ValidationDepartmentBlock<T>>::insert(department_id, storage_main_block);
				// check what if called again
				T::SchellingGameSharedSource::set_to_staking_period_pe_link(key.clone(), now)?;
				T::SchellingGameSharedSource::create_tree_helper_link(key, 3)?;

			//  println!("{:?}", data);
			} else {
				return Err(Error::<T>::CannotStakeNow.into());
			}

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn apply_jurors_positive_externality(
			origin: OriginFor<T>,
			department_id: DeparmentId,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Self::ensure_validation_on_positive_externality(user_to_calculate.clone())?;
			Self::ensure_min_stake_deparment(department_id)?;

			let pe_block_number = <ValidationDepartmentBlock<T>>::get(department_id);

			let key = SumTreeName::DepartmentScore {
				department_id,
				block_number: pe_block_number.clone(),
			};

			let game_type = SchellingGameType::DepartmentScore;

			T::SchellingGameSharedSource::apply_jurors_helper_link(key, game_type, who, stake)?;

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn pass_period(origin: OriginFor<T>, department_id: DeparmentId) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let pe_block_number = <ValidationDepartmentBlock<T>>::get(department_id);

			let key = SumTreeName::DepartmentScore {
				department_id,
				block_number: pe_block_number.clone(),
			};

			let now = <frame_system::Pallet<T>>::block_number();
			let game_type = SchellingGameType::DepartmentScore;
			T::SchellingGameSharedSource::change_period_link(key, game_type, now)?;

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn draw_jurors_positive_externality(
			origin: OriginFor<T>,
			department_id: DeparmentId,
			iterations: u64,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let pe_block_number = <ValidationDepartmentBlock<T>>::get(department_id);

			let key = SumTreeName::DepartmentScore {
				department_id,
				block_number: pe_block_number.clone(),
			};

			let game_type = SchellingGameType::DepartmentScore;

			T::SchellingGameSharedSource::draw_jurors_helper_link(key, game_type, iterations)?;

			Ok(())
		}

		// Unstaking
		// Stop drawn juror to unstake ✔️
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn unstaking(origin: OriginFor<T>, department_id: DeparmentId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let pe_block_number = <ValidationDepartmentBlock<T>>::get(department_id);

			let key = SumTreeName::DepartmentScore {
				department_id,
				block_number: pe_block_number.clone(),
			};

			T::SchellingGameSharedSource::unstaking_helper_link(key, who)?;
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn commit_vote(
			origin: OriginFor<T>,
			department_id: DeparmentId,
			vote_commit: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let pe_block_number = <ValidationDepartmentBlock<T>>::get(department_id);

			let key = SumTreeName::DepartmentScore {
				department_id,
				block_number: pe_block_number.clone(),
			};

			T::SchellingGameSharedSource::commit_vote_for_score_helper_link(key, who, vote_commit)?;
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn reveal_vote(
			origin: OriginFor<T>,
			department_id: DeparmentId,
			choice: i64,
			salt: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(choice <= 5 && choice >= 1, Error::<T>::ChoiceOutOfRange);

			let pe_block_number = <ValidationDepartmentBlock<T>>::get(department_id);

			let key = SumTreeName::DepartmentScore {
				department_id,
				block_number: pe_block_number.clone(),
			};

			T::SchellingGameSharedSource::reveal_vote_score_helper_link(key, who, choice, salt)?;
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn get_incentives(origin: OriginFor<T>, department_id: DeparmentId) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let pe_block_number = <ValidationDepartmentBlock<T>>::get(department_id);
			let key = SumTreeName::DepartmentScore {
				department_id,
				block_number: pe_block_number.clone(),
			};

			let game_type = SchellingGameType::DepartmentScore;
			T::SchellingGameSharedSource::get_incentives_score_schelling_helper_link(
				key.clone(),
				game_type,
				RangePoint::ZeroToFive,
			)?;

			let score = T::SchellingGameSharedSource::get_mean_value_link(key.clone());
			// // println!("Score {:?}", score);

			// To do
			// T::SharedStorageSource::set_positive_externality_link(user_to_calculate, score)?;

			Ok(())
		}
	}
}
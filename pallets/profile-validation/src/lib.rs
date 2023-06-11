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
mod permissions;
mod types;

use crate::types::{ChallengeEvidencePost, ChallengerFundInfo, ProfileFundInfo};
use frame_support::sp_runtime::traits::AccountIdConversion;
use frame_support::sp_runtime::SaturatedConversion;
use frame_support::sp_std::prelude::*;
use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_support::{
	traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons},
	PalletId,
};
use pallet_support::{
	ensure_content_is_valid, new_who_and_when, remove_from_vec, Content,
	WhoAndWhen, WhoAndWhenOf,
};
pub use types::{CitizenDetailsPost, FIRST_CITIZEN_ID, FIRST_CHALLENGE_POST_ID};
use schelling_game_shared::types::{Period, RangePoint, SchellingGameType, PhaseData};
use schelling_game_shared_link::SchellingGameSharedLink;
use sortition_sum_game::types::SumTreeName;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type ProfileFundInfoOf<T> =
	ProfileFundInfo<BalanceOf<T>, <T as frame_system::Config>::BlockNumber, AccountIdOf<T>>;
type ChallengerFundInfoOf<T> =
	ChallengerFundInfo<BalanceOf<T>, <T as frame_system::Config>::BlockNumber, AccountIdOf<T>>;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
type CitizenId = u64;
type ChallengePostId = u64;

const PALLET_ID: PalletId = PalletId(*b"ex/cfund");

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config + schelling_game_shared::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type SchellingGameSharedSource: SchellingGameSharedLink<
			SumTreeName = SumTreeName<Self::AccountId, Self::BlockNumber>,
			SchellingGameType = SchellingGameType,
			BlockNumber = Self::BlockNumber,
			AccountId = AccountIdOf<Self>,
			Balance = BalanceOf<Self>,
			RangePoint = RangePoint,
			Period = Period,
			PhaseData = PhaseData<Self>,
		>;
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);


	#[pallet::type_value]
	pub fn DefaultForNextCitizenId() -> CitizenId {
		FIRST_CITIZEN_ID
	}

	#[pallet::storage]
	#[pallet::getter(fn next_citizen_id)]
	pub type NextCitizenId<T> = StorageValue<_, CitizenId, ValueQuery, DefaultForNextCitizenId>;




	#[pallet::storage]
	#[pallet::getter(fn get_citizen_id)]
	pub type GetCitizenId<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, CitizenId>;

	#[pallet::storage]
	#[pallet::getter(fn citizen_profile)]
	pub type CitizenProfile<T> = StorageMap<_, Blake2_128Concat, CitizenId, CitizenDetailsPost<T>>; // Peer account id => Peer Profile Hash

	// Registration Fees

	#[pallet::type_value]
	pub fn DefaultRegistrationFee<T: Config>() -> BalanceOf<T> {
		1000u128.saturated_into::<BalanceOf<T>>()
	}
	// Registration challenge fees
	#[pallet::type_value]
	pub fn DefaultRegistrationChallengeFee<T: Config>() -> BalanceOf<T> {
		100u128.saturated_into::<BalanceOf<T>>()
	}

	#[pallet::storage]
	#[pallet::getter(fn profile_registration_fees)]
	pub type RegistrationFee<T> =
		StorageValue<_, BalanceOf<T>, ValueQuery, DefaultRegistrationFee<T>>;

	#[pallet::storage]
	#[pallet::getter(fn profile_registration_challenge_fees)]
	pub type RegistrationChallengeFee<T> =
		StorageValue<_, BalanceOf<T>, ValueQuery, DefaultRegistrationChallengeFee<T>>;

	#[pallet::storage]
	#[pallet::getter(fn profile_fund)]
	pub type ProfileFundDetails<T> =
		StorageMap<_, Blake2_128Concat, CitizenId, ProfileFundInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn challenger_fund)]
	pub type ChallengerFundDetails<T> =
		StorageMap<_, Blake2_128Concat, CitizenId, ChallengerFundInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn challenger_evidence_query)]
	pub type ChallengerEvidenceId<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, CitizenId, Blake2_128Concat, T::AccountId, ChallengePostId>; // profile number, challenger accountid => Challenge post id
	
		#[pallet::type_value]
	pub fn DefaultForNextChallengePostId() -> CitizenId {
		FIRST_CHALLENGE_POST_ID
	}
	
	#[pallet::storage]
	#[pallet::getter(fn next_challenge_post_count)]
	pub type NextChallengePostId<T> = StorageValue<_, ChallengePostId, ValueQuery, DefaultForNextChallengePostId>;

	#[pallet::storage]
	#[pallet::getter(fn challenge_post_comment)]
	pub type ChallengePostCommentIds<T> =
		StorageMap<_, Blake2_128Concat, ChallengePostId, Vec<ChallengePostId>, ValueQuery>; // challenge post id => Vec<Comment Post It>

	#[pallet::storage]
	#[pallet::getter(fn challenge_post)]
	pub type ChallengePost<T: Config> =
		StorageMap<_, Blake2_128Concat, ChallengePostId, ChallengeEvidencePost<T>>; // challenge post id => post



	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		CreateCitizen(T::AccountId, CitizenId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		ProfileExists,
		CitizenDoNotExists,
		ProfileFundExists,
		PostAlreadyExists,
		ProfileIsAlreadyValidated,
		ChallengeDoesNotExists,
		CommentExists,
		IsComment,
		ProfileFundNotExists,
		ChallengerFundInfoExists,
		NotProfileUser,
		NotEvidencePeriod,
		CitizenNotApproved,
		NotAPostOwner,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add citizen
		/// <pre>
		/// Get the count  
		/// Check that if who in exists in the GetCitizenId  
		///    if exists: ProfileExists error  
		///    if not exists: Insert the count in GetCitizenId  
		///                   Initialized CitizenDetails with who, count and ipfs profile_hash that contains profile data  
		///                   Insert count and citizen_details into CitizenProfile  
		///                   Increment citizen count and put the newcount to CitizenCount  
		///                   Release CreateCitizen event
		/// </pre>
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn add_citizen(origin: OriginFor<T>, content: Content) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let count = Self::next_citizen_id();
			match <GetCitizenId<T>>::get(&who) {
				Some(_citizen_id) => Err(Error::<T>::ProfileExists)?,
				None => {
					<GetCitizenId<T>>::insert(&who, count);

					let new_post: CitizenDetailsPost<T> = CitizenDetailsPost::new(count, who.clone(), content.clone());
			
					<CitizenProfile<T>>::insert(&count, new_post);
					NextCitizenId::<T>::mutate(|n| {
						*n += 1;
					});
					Self::deposit_event(Event::CreateCitizen(who, count));
					Ok(())
				},
			}
		}

		/// Add profile fund  
		/// <pre>
		/// Check profile exists for profile_citizenid, if not throw error CitizenDoNotExists inside the function get_citizen_accountid
		/// Get the RegistrationFee and store in deposit variable
		/// Get the current block number
		/// Withdraw the deposit or RegistrationFee
		/// Check the profile_citizenid exists in ProfileFundDetails:
		///        if exists: ProfileFundExists error
		///        if doesnot exits:
		///                          create profile fund info with who, deposit, block_time,
		///                          Insert profile_fund_info into ProfileFundDetails
		/// Create sortition sum tree
		/// Set the evidence period to now
		/// Enhancement:
		/// How to stake for profile?
		/// For profile validation should one person submit the staking fee, or allow crowdfunding. What will be better?
		/// </pre>

		/// Update profile
		/// Can update profile if evidence period is not over

		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		// pub fn update_profile(
		// 	origin: OriginFor<T>,
		// 	profile_citizenid: u128,
		// 	profile_hash: Vec<u8>,
		// ) -> DispatchResult {
		// 	let who = ensure_signed(origin)?;
		// 	let citizen_account_id = Self::get_citizen_accountid(profile_citizenid)?;
		// 	ensure!(who == citizen_account_id, Error::<T>::NotProfileUser);

		// 	let citizen_details = CitizenDetails {
		// 		profile_hash: profile_hash.clone(),
		// 		citizenid: profile_citizenid,
		// 		accountid: who.clone(),
		// 	};

		// 	let key = SumTreeName::UniqueIdenfier1 {
		// 		citizen_id: profile_citizenid,
		// 		name: "challengeprofile".as_bytes().to_vec(),
		// 	};

		// 	let period = T::SchellingGameSharedSource::get_period_link(key);
		// 	match period {
		// 		None => {
		// 			<CitizenProfile<T>>::insert(&profile_citizenid, citizen_details);
		// 		},
		// 		Some(periodname) => {
		// 			ensure!(periodname == Period::Evidence, Error::<T>::NotEvidencePeriod);
		// 			<CitizenProfile<T>>::insert(&profile_citizenid, citizen_details);
		// 		},
		// 	}

		// 	Ok(())
		// }

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn add_profile_fund(origin: OriginFor<T>, profile_citizenid: CitizenId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let _citizen_account_id = Self::get_citizen_accountid(profile_citizenid)?;
			let deposit = <RegistrationFee<T>>::get();
			let now = <frame_system::Pallet<T>>::block_number();

			let imb = <T as pallet::Config>::Currency::withdraw(
				&who,
				deposit,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?;

			<T as pallet::Config>::Currency::resolve_creating(&Self::fund_profile_account(), imb);

			match <ProfileFundDetails<T>>::get(&profile_citizenid) {
				// 📝 To write update stake for reapply when disapproved
				Some(_profilefundinfo) => Err(Error::<T>::ProfileFundExists)?,
				None => {
					let profile_fund_info = ProfileFundInfo {
						funder_account_id: who,
						deposit,
						start: now.clone(),
						validated: false,
						reapply: false,
						deposit_returned: false,
					};
					<ProfileFundDetails<T>>::insert(&profile_citizenid, profile_fund_info);
				},
			}

			let key = SumTreeName::UniqueIdenfier1 {
				citizen_id: profile_citizenid,
				name: "challengeprofile".as_bytes().to_vec(),
			};

			let result = T::SchellingGameSharedSource::set_to_evidence_period_link(key, now);

			result
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn challenge_evidence(
			origin: OriginFor<T>,
			profile_citizenid: CitizenId,
			content: Content,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let citizen_account_id = Self::get_citizen_accountid(profile_citizenid)?;
			let count = Self::next_challenge_post_count();
			let challenge_evidence_post = ChallengeEvidencePost::new(citizen_account_id, who.clone(), content, None); 
			match <ChallengerEvidenceId<T>>::get(&profile_citizenid, &who) {
				None => {
					<ChallengePost<T>>::insert(&count, challenge_evidence_post);
					NextChallengePostId::<T>::mutate(|n| {
						*n += 1;
					});

					<ChallengerEvidenceId<T>>::insert(&profile_citizenid, &who, count);
				},
				Some(_hash) => {
					Err(Error::<T>::PostAlreadyExists)?
					// match <ChallengerFundDetails<T>>::get(&profile_citizenid) {
					// 	Some(_challengerfundinfo) => {
					// 		Err(Error::<T>::ChallengerFundAddedCanNotUpdate)?
					// 	},
					// 	None => {
					// 		// Update challenger profile
					// 		<ChallengePost<T>>::insert(&count, challenge_evidence_post);
					// 		let newcount =
					// 			count.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// 		<ChallengePostCount<T>>::put(newcount);
					// 		<ChallengerEvidenceId<T>>::insert(&profile_citizenid, &who, count);
					// 	},
					// }
				},
			}
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn challenge_comment_create(
			origin: OriginFor<T>,
			post_id: ChallengePostId,
			content: Content,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let count = Self::next_challenge_post_count();
			let main_evidence_post = Self::challenge_post(post_id).unwrap();
			let challenge_evidence_post = ChallengeEvidencePost::new(main_evidence_post.kyc_profile_id, who, content, Some(post_id)); 

			match <ChallengePost<T>>::get(&post_id) {
				None => Err(Error::<T>::ChallengeDoesNotExists)?,
				Some(challenge_evidence_post_c) => {
					if challenge_evidence_post_c.is_comment == false {
						<ChallengePost<T>>::insert(&count, challenge_evidence_post);
						NextChallengePostId::<T>::mutate(|n| {
							*n += 1;
						});
						let mut comment_ids = <ChallengePostCommentIds<T>>::get(&post_id);
						match comment_ids.binary_search(&count) {
							Ok(_) => Err(Error::<T>::CommentExists)?,
							Err(index) => {
								comment_ids.insert(index, count.clone());
								<ChallengePostCommentIds<T>>::insert(&post_id, &comment_ids);
							},
						}
					} else {
						Err(Error::<T>::IsComment)?
					}
				},
			}

			Ok(())
		}

		// Does citizen exists ✔️
		// Has the citizen added profile fund ✔️
		// Create tree ✔️
		// Check evidence has been submitted
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn challenge_profile(origin: OriginFor<T>, profile_citizenid: CitizenId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let key = SumTreeName::UniqueIdenfier1 {
				citizen_id: profile_citizenid,
				name: "challengeprofile".as_bytes().to_vec(),
			};
			let phase_data = Self::get_phase_data();
			let now = <frame_system::Pallet<T>>::block_number();
			let _citizen_account_id = Self::get_citizen_accountid(profile_citizenid)?;
			match <ProfileFundDetails<T>>::get(&profile_citizenid) {
				Some(profilefundinfo) => {
					if profilefundinfo.validated == true {
						Err(Error::<T>::ProfileIsAlreadyValidated)?;
					} else {
						let _evidence_stake_block_number = profilefundinfo.start; // remove the profile fund info start

				  T::SchellingGameSharedSource::set_to_staking_period_link(
							key.clone(),
							phase_data,
							now,
						)?;
					}
				},
				None => {
					Err(Error::<T>::ProfileFundNotExists)?;
				},
			}
			let deposit = <RegistrationChallengeFee<T>>::get();
			let imb = <T as pallet::Config>::Currency::withdraw(
				&who,
				deposit,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?;

			<T as pallet::Config>::Currency::resolve_creating(&Self::fund_profile_account(), imb);

			match <ChallengerFundDetails<T>>::get(&profile_citizenid) {
				// 📝 To write update stake for reapply
				Some(_challengerfundinfo) => Err(Error::<T>::ChallengerFundInfoExists)?,
				None => {
					let challenger_fund_info = ChallengerFundInfo {
						challengerid: who,
						deposit,
						start: now,
						challenge_completed: false,
					};
					<ChallengerFundDetails<T>>::insert(&profile_citizenid, challenger_fund_info);
				},
			}

			let result = T::SchellingGameSharedSource::create_tree_helper_link(key, 3);
			result
		}

		// May be you need to check challeger fund details exists
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn pass_period(origin: OriginFor<T>, profile_citizenid: CitizenId) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let key = SumTreeName::UniqueIdenfier1 {
				citizen_id: profile_citizenid,
				name: "challengeprofile".as_bytes().to_vec(),
			};

			let now = <frame_system::Pallet<T>>::block_number();
			let phase_data = Self::get_phase_data();

			let result = T::SchellingGameSharedSource::change_period_link(key, phase_data, now);

			result
		}

		// To Do
		// Apply jurors or stake ✔️
		// Update stake
		// Draw jurors ✔️
		// Unstaking non selected jurors ✔️
		// Commit vote ✔️
		// Reveal vote ✔️
		// Get winning decision ✔️
		// Incentive distribution ✔️

		// Staking
		// 1. Check for minimum stake ✔️
		// 2. Check period is Staking ✔️
		// 3. Number of people staked

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn apply_jurors(
			origin: OriginFor<T>,
			profile_citizenid: CitizenId,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let key = SumTreeName::UniqueIdenfier1 {
				citizen_id: profile_citizenid,
				name: "challengeprofile".as_bytes().to_vec(),
			};

			let phase_data = Self::get_phase_data();

			let result =
				T::SchellingGameSharedSource::apply_jurors_helper_link(key, phase_data, who, stake);
			result
		}

		// Draw jurors
		// Check period is drawing ✔️
		// Check mininum number of juror staked ✔️
		// Improvements
		// Set stake to zero so that they are not drawn again
		// Store the drawn juror stake in hashmap storage
		// Add min draws along with max draws
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn draw_jurors(
			origin: OriginFor<T>,
			profile_citizenid: CitizenId,
			iterations: u64,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let key = SumTreeName::UniqueIdenfier1 {
				citizen_id: profile_citizenid,
				name: "challengeprofile".as_bytes().to_vec(),
			};
			let phase_data = Self::get_phase_data();

			let result =
				T::SchellingGameSharedSource::draw_jurors_helper_link(key, phase_data, iterations);
			result
		}

		// Unstaking
		// Stop drawn juror to unstake ✔️
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn unstaking(origin: OriginFor<T>, profile_citizenid: CitizenId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let key = SumTreeName::UniqueIdenfier1 {
				citizen_id: profile_citizenid,
				name: "challengeprofile".as_bytes().to_vec(),
			};
			let result = T::SchellingGameSharedSource::unstaking_helper_link(key, who);
			result
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn commit_vote(
			origin: OriginFor<T>,
			profile_citizenid: CitizenId,
			vote_commit: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let key = SumTreeName::UniqueIdenfier1 {
				citizen_id: profile_citizenid,
				name: "challengeprofile".as_bytes().to_vec(),
			};
			let result =
				T::SchellingGameSharedSource::commit_vote_helper_link(key, who, vote_commit);
			result
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn reveal_vote(
			origin: OriginFor<T>,
			profile_citizenid: CitizenId,
			choice: u128,
			salt: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let key = SumTreeName::UniqueIdenfier1 {
				citizen_id: profile_citizenid,
				name: "challengeprofile".as_bytes().to_vec(),
			};

			let result = T::SchellingGameSharedSource::reveal_vote_two_choice_helper_link(
				key, who, choice, salt,
			);
			result
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn get_incentives(origin: OriginFor<T>, profile_citizenid: CitizenId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let key = SumTreeName::UniqueIdenfier1 {
				citizen_id: profile_citizenid,
				name: "challengeprofile".as_bytes().to_vec(),
			};
			let phase_data = Self::get_phase_data();
			let result = T::SchellingGameSharedSource::get_incentives_two_choice_helper_link(
				key, phase_data, who,
			);
			result
		}

		// Write code for return profile incentives
	}
}

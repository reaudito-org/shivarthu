#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;

#[frame_support::pallet]
pub mod pallet {
	use crate::types::{
		CitizenDetails, DepartmentDetails, ProfileFundInfo, SchellingType, SortitionSumTree,
		StakeDetails,
	};
	use frame_support::sp_runtime::traits::AccountIdConversion;
	use frame_support::sp_runtime::traits::CheckedSub;
	use frame_support::sp_runtime::SaturatedConversion;
	use frame_support::sp_std::{collections::btree_map::BTreeMap, vec::Vec};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_support::{sp_runtime::app_crypto::sp_core::H256, traits::Randomness};
	use frame_support::{
		traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	type ProfileFundInfoOf<T> =
		ProfileFundInfo<BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;
	type CitizenDetailsOf<T> = CitizenDetails<AccountIdOf<T>>;

	type FundIndex = u32;

	const PALLET_ID: PalletId = PalletId(*b"ex/cfund");

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: ReservableCurrency<Self::AccountId>;

		type RandomnessSource: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	pub type Nonce<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn department_count)]
	pub type DepartmentCount<T> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn admin)]
	pub type Admin<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn citizen_count)]
	pub type CitizenCount<T> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn citizen_id)]
	pub type CitizenId<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u128>;

	#[pallet::storage]
	#[pallet::getter(fn citizen_profile)]
	pub type CitizenProfile<T> = StorageMap<_, Blake2_128Concat, u128, CitizenDetailsOf<T>>; // Peer account id => Peer Profile Hash

	// Registration Fees

	#[pallet::type_value]
	pub fn DefaultRegistrationFees<T: Config>() -> BalanceOf<T> {
		100u128.saturated_into::<BalanceOf<T>>()
	}

	#[pallet::storage]
	#[pallet::getter(fn profile_registration_fees)]
	pub type RegistrationFee<T> =
		StorageValue<_, BalanceOf<T>, ValueQuery, DefaultRegistrationFees<T>>;

	#[pallet::storage]
	#[pallet::getter(fn profile_fund)]
	pub type FundProfileDetails<T> = StorageMap<_, Blake2_128Concat, u128, ProfileFundInfoOf<T>>;

	// #[pallet::storage]
	// #[pallet::getter(fn citizen_profile_status)]

	#[pallet::storage]
	#[pallet::getter(fn department_profile)]
	pub type Department<T> = StorageMap<_, Blake2_128Concat, u128, DepartmentDetails>; // Deparment id => (Name, Location, Details hash)

	#[pallet::storage]
	#[pallet::getter(fn outergroup)]
	pub type OuterGroup<T> = StorageMap<_, Blake2_128Concat, u128, Vec<u128>>; // Department id => Candidate account address set

	#[pallet::storage]
	#[pallet::getter(fn innergroup)]
	pub type InnerGroup<T> = StorageMap<_, Blake2_128Concat, u128, Vec<u128>>; // Department id => Candidate account address set

	#[pallet::storage]
	#[pallet::getter(fn fullgroup)]
	pub type FullGroup<T> = StorageMap<_, Blake2_128Concat, u128, Vec<u128>>; // Department id => Candidate account address set

	#[pallet::storage]
	#[pallet::getter(fn citizen_departments)]
	pub type CitizenDepartments<T> = StorageMap<_, Blake2_128Concat, u128, Vec<u128>>; // Peer account address => Department id set

	#[pallet::storage]
	#[pallet::getter(fn governor_group)]
	pub type GovernorGroup<T> = StorageMap<_, Blake2_128Concat, u128, Vec<u128>>; // Department id => Candidate account address set

	#[pallet::storage]
	#[pallet::getter(fn candidate_nominee)]
	pub type CandidateNominees<T> = StorageMap<_, Blake2_128Concat, (u128, u128), Vec<u128>>; // Department id, Voting cycle => Candidate account address set

	#[pallet::storage]
	#[pallet::getter(fn candidate_approval_votes)]
	pub type CandidateApprovalVotes<T> = StorageMap<_, Blake2_128Concat, (u128, u128, u128), u128>; // Candidate account address, Department id, voting cycle=> Positive Votes

	// Schelling Game Storage

	#[pallet::storage]
	#[pallet::getter(fn schelling_stake)]
	pub type SchellingStake<T> = StorageDoubleMap<
		_,
		Twox64Concat,
		u128,
		Twox64Concat,
		SchellingType,
		StakeDetails<BalanceOf<T>>,
	>; // (citizen id, schelling type => stake)

	#[pallet::storage]
	#[pallet::getter(fn sortition_sum_trees)]
	pub type SortitionSumTrees<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, SortitionSumTree>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		CreateDepartment(u128, T::AccountId),
		CitizenDepartment(u128, T::AccountId),
		CreateCitizen(T::AccountId, Vec<u8>),
		VoteCast(u128, u128, u128), // Departement id, cycle, department vote count
		NomineeDeparment(u128, u128, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		InvalidIndex,
		FailedUnwrap,

		DepartmentExists,
		DepartmentDoNotExists,
		DepartmentNotAssociated,
		ProfileExists,
		ProfileFundExists,
		NomineeExists,
		CitizenDoNotExists,
		AlreadyCommitUsed,
		VoteAlreadyRevealed,
		VoteCommitNotPresent,
		CommitVoteMismatch,
		ProfileNotFunded,
		ProfileValidationOver,
		AlreadyStaked,
		ApplyJurorTimeNotEnded,
		KMustGreaterThanOne,
		TreeAlreadyExists,
		TreeDoesnotExist,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn add_citizen(origin: OriginFor<T>, profile_hash: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let count = <CitizenCount<T>>::get();
			match <CitizenId<T>>::get(&who) {
				Some(_citizen_id) => Err(Error::<T>::ProfileExists)?,
				None => {
					<CitizenId<T>>::insert(&who, count);
					let citizen_details = CitizenDetails {
						profile_hash: profile_hash.clone(),
						citizenid: count,
						accountid: who.clone(),
					};
					<CitizenProfile<T>>::insert(&count, citizen_details);
					let newcount = count.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					<CitizenCount<T>>::put(newcount);
					Self::deposit_event(Event::CreateCitizen(who, profile_hash));
					Ok(())
				}
			}
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn add_profile_fund(origin: OriginFor<T>, citizenid: u128) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let _citizen_account_id = Self::get_citizen_accountid(citizenid)?;
			let deposit = <RegistrationFee<T>>::get();
			let now = <frame_system::Pallet<T>>::block_number();

			let imb = T::Currency::withdraw(
				&who,
				deposit,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?;

			T::Currency::resolve_creating(&Self::fund_profile(), imb);

			match <FundProfileDetails<T>>::get(&citizenid) {
				// 📝 To write update stake for reapply when disapproved
				Some(_profilefundinfo) => Err(Error::<T>::ProfileExists)?,
				None => {
					let profile_fund_info =
						ProfileFundInfo { deposit, start: now, validated: false, reapply: false };
					<FundProfileDetails<T>>::insert(&citizenid, profile_fund_info);
				}
			}

			Ok(())
		}

		// Generic Schelling game
		// 1. Adding to SchellingStake ✔️
		// 2. Check for minimum stake ❌
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn apply_jurors(
			origin: OriginFor<T>,
			schellingtype: SchellingType,
			stake: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let who_citizen_id = Self::get_citizen_id(who)?;
			let stake_info = StakeDetails { stake };
			match schellingtype {
				SchellingType::ProfileApproval { citizen_id } => {
					let _profile_added = Self::profile_fund_added(citizen_id);
					match <SchellingStake<T>>::get(&who_citizen_id, &schellingtype) {
						Some(_stake) => Err(Error::<T>::AlreadyStaked)?,
						None => {
							<SchellingStake<T>>::insert(
								&who_citizen_id,
								&schellingtype,
								stake_info,
							);
						}
					}
					Ok(())
				}
			}
		}

		// Draw jurors
		// Check whether juror application time is over, if not throw error
		// Check mininum number of juror staked
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn draw_jurors(origin: OriginFor<T>, schellingtype: SchellingType) -> DispatchResult {
			let now = <frame_system::Pallet<T>>::block_number();
			match schellingtype {
				SchellingType::ProfileApproval { citizen_id } => {
					let profilefundinfo = Self::get_profile_fund_info(citizen_id)?;
					let start_block = profilefundinfo.start;
					let data = now.checked_sub(&start_block).unwrap();
					if data < 432000u128.saturated_into::<BlockNumberFor<T>>() {
						Err(Error::<T>::ApplyJurorTimeNotEnded)?
					}
				}
			}
			Ok(())
		}

		// SortitionSumTree
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn create_tree(origin: OriginFor<T>, key: Vec<u8>, k: u64) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if k < 1 {
				Err(Error::<T>::KMustGreaterThanOne)?
			}
			let tree_option = <SortitionSumTrees<T>>::get(&key);
			match tree_option {
				Some(_tree) => Err(Error::<T>::TreeAlreadyExists)?,
				None => {
					let mut sum_tree = SortitionSumTree {
						k,
						stack: Vec::new(),
						nodes: Vec::new(),
						ids_to_node_indexes: BTreeMap::new(),
						node_indexes_to_ids: BTreeMap::new(),
					};

					sum_tree.nodes.push(0);

					<SortitionSumTrees<T>>::insert(&key, &sum_tree);
				}
			}
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,2))]
		pub fn set(
			origin: OriginFor<T>,
			key: Vec<u8>,
			value: u64,
			citizen_id: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let tree_option = <SortitionSumTrees<T>>::get(&key);

			match tree_option {
				None => Err(Error::<T>::TreeDoesnotExist)?,
				Some(mut tree) => match tree.ids_to_node_indexes.get(&citizen_id) {
					Some(tree_index_data) => {
						let tree_index = *tree_index_data;
						if tree_index == 0 {
							Self::if_tree_index_zero(value, citizen_id, tree, tree_index, key);
						} else {
							// Existing node
							if value == 0 {
								let value = tree.nodes[tree_index as usize];
								tree.nodes[tree_index as usize] = 0;
								tree.stack.push(tree_index);
								tree.ids_to_node_indexes.remove(&citizen_id);
								tree.node_indexes_to_ids.remove(&tree_index);

								// UpdateParents 🟥
								Self::update_parents(tree, tree_index, false, value, key);
							} else if value != tree.nodes[tree_index as usize] {
								let plus_or_minus = tree.nodes[tree_index as usize] <= value;
								let plus_or_minus_value = if plus_or_minus {
									value - tree.nodes[tree_index as usize]
								} else {
									tree.nodes[tree_index as usize] - value
								};
								tree.nodes[tree_index as usize] = value;

								// update parents 🟥
								Self::update_parents(
									tree,
									tree_index,
									plus_or_minus,
									plus_or_minus_value,
									key,
								);
							}
						}
					}

					None => {
						Self::if_tree_index_zero(value, citizen_id, tree, 0, key);
					}
				},
			}

			Ok(())
		}

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
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
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				}
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_citizen_accountid(citizenid: u128) -> Result<T::AccountId, DispatchError> {
			let profile = Self::citizen_profile(citizenid).ok_or(Error::<T>::InvalidIndex)?;
			Ok(profile.accountid)
		}

		fn get_citizen_id(accountid: T::AccountId) -> Result<u128, DispatchError> {
			match Self::citizen_id(accountid) {
				Some(citizen_id) => Ok(citizen_id),
				None => Err(Error::<T>::ProfileNotFunded)?,
			}
		}

		fn profile_fund_added(citizenid: u128) -> DispatchResult {
			match <FundProfileDetails<T>>::get(&citizenid) {
				Some(profilefundinfo) => {
					let validated = profilefundinfo.validated;
					let reapply = profilefundinfo.reapply;
					if validated == false && reapply == false {
						Ok(())
					} else {
						Err(Error::<T>::ProfileValidationOver)?
					}
				}
				None => Err(Error::<T>::ProfileNotFunded)?,
			}
		}

		fn get_profile_fund_info(citizenid: u128) -> Result<ProfileFundInfoOf<T>, DispatchError> {
			match <FundProfileDetails<T>>::get(&citizenid) {
				Some(profilefundinfo) => {
					let validated = profilefundinfo.validated;
					let reapply = profilefundinfo.reapply;
					if validated == false && reapply == false {
						Ok(profilefundinfo)
					} else {
						Err(Error::<T>::ProfileValidationOver)?
					}
				}
				None => Err(Error::<T>::ProfileNotFunded)?,
			}
		}

		fn fund_profile() -> T::AccountId {
			PALLET_ID.into_sub_account(1)
		}

		fn draw_juror_for_citizen_profile_function(
			citizen_id: u128,
			length: usize,
		) -> DispatchResult {
			let nonce = Self::get_and_increment_nonce();

			let random_seed = T::RandomnessSource::random(&nonce).encode();
			let random_number = u64::decode(&mut random_seed.as_ref())
				.expect("secure hashes should always be bigger than u64; qed");
			Ok(())
		}

		fn get_and_increment_nonce() -> Vec<u8> {
			let nonce = <Nonce<T>>::get();
			<Nonce<T>>::put(nonce.wrapping_add(1));
			nonce.encode()
		}

		fn update_parents(
			mut tree: SortitionSumTree,
			tree_index: u64,
			plus_or_minus: bool,
			value: u64,
			key: Vec<u8>,
		) {
			let mut parent_index = tree_index;
			while parent_index != 0 {
				parent_index = (parent_index - 1) / tree.k;
				tree.nodes[parent_index as usize] = if plus_or_minus {
					tree.nodes[parent_index as usize] + value
				} else {
					tree.nodes[parent_index as usize] - value
				};
				<SortitionSumTrees<T>>::insert(&key, &tree);
			}
		}
		fn if_tree_index_zero(
			value: u64,
			citizen_id: u128,
			mut tree: SortitionSumTree,
			mut tree_index: u64,
			key: Vec<u8>,
		) {
			// No existing node.
			if value != 0 {
				// Non zero value.
				// Append.
				// Add node.
				if tree.stack.len() == 0 {
					// No vacant spots.
					// Get the index and append the value.
					tree_index = tree.nodes.len() as u64;
					tree.nodes.push(value);

					// println!("{}", tree_index);

					// Potentially append a new node and make the parent a sum node.
					if tree_index != 1 && (tree_index - 1) % tree.k == 0 {
						// Is first child.
						let parent_index = tree_index / tree.k;
						let parent_id = *tree.node_indexes_to_ids.get(&parent_index).unwrap();
						let new_index = tree_index + 1;
						tree.nodes.push(*tree.nodes.get(parent_index as usize).unwrap());
						tree.node_indexes_to_ids.remove(&parent_index);
						tree.ids_to_node_indexes.insert(parent_id, new_index);
						tree.node_indexes_to_ids.insert(new_index, parent_id);
					}
				} else {
					let tree_index = tree.stack.get(tree.stack.len() - 1);
					tree.nodes[*tree_index.unwrap() as usize] = value;
					tree.stack.pop();
				}

				tree.ids_to_node_indexes.insert(citizen_id, tree_index);
				tree.node_indexes_to_ids.insert(tree_index, citizen_id);

				// update_parents 🟥

				Self::update_parents(tree, tree_index, true, value, key);
			}
		}

		pub fn stake_of(key: Vec<u8>, citizen_id: u128) -> Result<u64, DispatchError> {
			let tree_option = <SortitionSumTrees<T>>::get(&key);
			match tree_option {
				None => Err(Error::<T>::TreeDoesnotExist)?,
				Some(tree) => {
					let tree_index_data = tree.ids_to_node_indexes.get(&citizen_id).unwrap();

					let value: u64;
					let tree_index = *tree_index_data;
					if tree_index == 0 {
						value = 0;
					} else {
						value = tree.nodes[tree_index as usize];
					}
					Ok(value)
				}
			}
		}

		pub fn draw(key: Vec<u8>, draw_number: u64) -> Result<u128, DispatchError> {
			let tree_option = <SortitionSumTrees<T>>::get(&key);

			match tree_option {
				None => Err(Error::<T>::TreeDoesnotExist)?,
				Some(tree) => {
					let mut tree_index = 0;
					let mut current_draw_number = draw_number % tree.nodes[0];

					while (tree.k * tree_index) + 1 < (tree.nodes.len() as u64) {
						for i in 1..tree.k + 1 {
							let node_index = (tree.k * tree_index) + i;
							let node_value = tree.nodes[node_index as usize];

							if current_draw_number >= node_value {
								current_draw_number -= node_value;
							} else {
								tree_index = node_index;
								break;
							}
						}
					}

					Ok(*tree.node_indexes_to_ids.get(&tree_index).unwrap())
				}
			}
		}
	}
}

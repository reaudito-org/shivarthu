use frame_support::{pallet_prelude::*};
use frame_support::sp_std::{vec::Vec, collections::btree_map::BTreeMap};
use scale_info::TypeInfo;
use frame_support::traits::ConstU32;


#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct DepartmentDetails {
    pub name: Vec<u8>,
    pub location: Vec<u8>,
    pub details: Vec<u8>,
    pub departmentid: u128,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct CitizenDetails<AccountId> {
    pub profile_hash: Vec<u8>,
    pub citizenid: u128,
    pub accountid: AccountId,
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ProfileFundInfo<Balance, BlockNumber, AccountId> {
    pub funder_account_id: AccountId,
    pub deposit: Balance,
    pub start: BlockNumber,
    pub validated: bool,
    pub reapply: bool,
    pub deposit_returned:bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ChallengerFundInfo<Balance, BlockNumber, AccountId> {
    pub challengerid: AccountId,
    pub deposit: Balance,
    pub start: BlockNumber,
    pub challenge_completed: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode,MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum SchellingType {
    ProfileApproval{ citizen_id: u128 }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct StakeDetails<Balance> {
    pub stake: Balance,
}


// #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, TypeInfo)]
// #[cfg_attr(feature = "std", derive(Debug))]
// pub enum Phase {
//     Staking, // Stake sum trees can be updated. Pass after `minStakingTime` passes and there is at least one dispute without jurors.
//     Generating, // Waiting for a random number. Pass as soon as it is ready.
//     Drawing // Jurors can be drawn. Pass after all disputes have jurors or `maxDrawingTime` passes.
//   }

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, MaxEncodedLen,TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Period {
    Evidence, // Evidence can be submitted. This is also when drawing has to take place.
    Staking, // Stake sum trees can be updated. Pass after `minStakingTime` passes and there is at least one dispute without jurors.
    Commit, // Jurors commit a hashed vote. This is skipped for courts without hidden votes.
    Vote, // Jurors reveal/cast their vote depending on whether the court has hidden votes or not.
    Appeal, // The dispute can be appealed.
    Drawing, // Jurors can be drawn. Pass after all disputes have jurors or `maxDrawingTime` passes.
    Execution // Tokens are redistributed and the ruling is executed.
  }

  #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
  #[cfg_attr(feature = "std", derive(Debug))]
  pub struct DrawJurorsForProfileLimit {
      pub max_draws: u64,
      pub max_draws_appeal: u64
  }

  #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode,MaxEncodedLen, TypeInfo)]
  #[cfg_attr(feature = "std", derive(Debug))]
  pub struct StakingTime<BlockNumber> {
      pub min_short_block_length: BlockNumber,
      pub min_long_block_length: BlockNumber,
  }

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode,MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum VoteStatus {
    Commited,
    Revealed
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode,MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct CommitVote {
    pub commit: [u8; 32],
    pub votestatus: VoteStatus,
    pub vote_revealed: Option<u8>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ChallengeEvidencePost<AccountId> {
    pub author_account_id: AccountId,
    pub post_hash: Vec<u8>,
    pub is_comment: bool,
}













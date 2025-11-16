use codec::{Decode, Encode, MaxEncodedLen};
use frame::{deps::frame_support::pallet_prelude::*, prelude::H256};
use scale_info::TypeInfo;

use crate::constants::{MAX_DESCRIPTION_LENGTH, MAX_URL_LENGTH};

pub type Description = BoundedVec<u8, ConstU32<MAX_DESCRIPTION_LENGTH>>;
pub type DocumentURL = BoundedVec<u8, ConstU32<MAX_URL_LENGTH>>;

#[derive(
    Clone,
    Encode,
    Decode,
    Eq,
    PartialEq,
    RuntimeDebug,
    MaxEncodedLen,
    TypeInfo,
    DecodeWithMemTracking,
)]
pub enum CaseType<AccountId> {
    HighRiskAddress { account: AccountId },
    HighRiskTx { tx_hash: H256 },
    CleanAddress { account: AccountId },
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Case<AccountId, BlockNumber> {
    pub id: u128,
    pub submitter: AccountId,
    pub case_type: CaseType<AccountId>,
    pub document_url: DocumentURL,
    pub description: Description,
    pub submitted_at: BlockNumber,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum CaseStatus {
    Open,
    Accepted,
    Rejected,
    Inconclusive,
}

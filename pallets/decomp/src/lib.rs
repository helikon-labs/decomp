#![cfg_attr(not(feature = "std"), no_std)]

//! Decomp pallet - initial implementation.

pub use pallet::*;

mod constants;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod types;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame::pallet]
pub mod pallet {
    use frame::prelude::*;

    use crate::types::{Case, CaseStatus, CaseType, Description, DocumentURL};

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: crate::weights::WeightInfo;
        #[pallet::constant]
        type ChallengeWindow: Get<BlockNumberFor<Self>>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Total case count.
    #[pallet::storage]
    pub(super) type CaseCount<T: Config> = StorageValue<_, u128, ValueQuery>;

    /// All cases by id.
    #[pallet::storage]
    pub(super) type Cases<T: Config> =
        StorageMap<_, Blake2_128Concat, u128, Case<T::AccountId, BlockNumberFor<T>>>;

    /// Case statuses.
    #[pallet::storage]
    pub type CaseStatusOf<T: Config> = StorageMap<_, Blake2_128Concat, u128, CaseStatus>;

    /// Case deadlines (block #).
    #[pallet::storage]
    pub type CaseDeadline<T: Config> = StorageMap<_, Blake2_128Concat, u128, BlockNumberFor<T>>;

    /// Supporters of a case.
    #[pallet::storage]
    pub type CaseSupporters<T: Config> =
        StorageMap<_, Blake2_128Concat, u128, BoundedVec<T::AccountId, ConstU32<256>>, ValueQuery>;

    /// Challengers of a case.
    #[pallet::storage]
    pub type CaseChallengers<T: Config> =
        StorageMap<_, Blake2_128Concat, u128, BoundedVec<T::AccountId, ConstU32<256>>, ValueQuery>;

    /// Cases by deadline - for easy processing by block number.
    #[pallet::storage]
    pub type CasesByDeadline<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<u128, ConstU32<1024>>,
        ValueQuery,
    >;

    /// Event definitions.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CaseSubmitted {
            id: u128,
            who: T::AccountId,
        },
        CaseSupported {
            id: u128,
            who: T::AccountId,
        },
        CaseChallenged {
            id: u128,
            who: T::AccountId,
        },
        CaseAccepted {
            id: u128,
            support_count: u32,
            challenge_count: u32,
        },
        CaseRejected {
            id: u128,
            support_count: u32,
            challenge_count: u32,
        },
        CaseInconclusive {
            id: u128,
            support_count: u32,
            challenge_count: u32,
        },
    }

    /// Error definitions.
    #[pallet::error]
    pub enum Error<T> {
        StorageOverflow,
        AlreadySupporting,
        AlreadyChallenging,
        UnknownCase,
        TooEarlyToFinalize,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            log::info!("Check block {now:?} for deadlines.",);
            let cases = CasesByDeadline::<T>::take(now);
            log::info!(
                "There are {} cases with deadlines in block {now:?}.",
                cases.len()
            );
            for case_id in cases.iter() {
                log::info!("Finalize case {case_id}.");
                let support_count = CaseSupporters::<T>::get(case_id).len() as u32;
                let challenge_count = CaseChallengers::<T>::get(case_id).len() as u32;
                if support_count > challenge_count {
                    CaseStatusOf::<T>::insert(case_id, CaseStatus::Accepted);
                    Self::deposit_event(Event::CaseAccepted {
                        id: *case_id,
                        support_count,
                        challenge_count,
                    });
                } else if challenge_count > support_count {
                    CaseStatusOf::<T>::insert(case_id, CaseStatus::Rejected);
                    Self::deposit_event(Event::CaseRejected {
                        id: *case_id,
                        support_count,
                        challenge_count,
                    });
                } else {
                    CaseStatusOf::<T>::insert(case_id, CaseStatus::Inconclusive);
                    Self::deposit_event(Event::CaseInconclusive {
                        id: *case_id,
                        support_count,
                        challenge_count,
                    });
                }
            }
            T::DbWeight::get().reads_writes(cases.len() as u64 * 3, cases.len() as u64 * 2)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(2, 2))]
        #[allow(clippy::useless_conversion)]
        pub fn submit_case(
            origin: OriginFor<T>,
            case_type: CaseType<T::AccountId>,
            document_url: DocumentURL,
            description: Description,
        ) -> DispatchResultWithPostInfo {
            let submitter = ensure_signed(origin)?;
            let old_case_count = CaseCount::<T>::get();
            let case_id = old_case_count;
            let new_case_count = old_case_count
                .checked_add(One::one())
                .ok_or(Error::<T>::StorageOverflow)?;
            let submitted_at: BlockNumberFor<T> = <frame_system::Pallet<T>>::block_number();
            let case = Case {
                id: case_id,
                submitter: submitter.clone(),
                case_type,
                document_url,
                description,
                submitted_at,
            };
            Cases::<T>::insert(case_id, case);
            CaseCount::<T>::set(new_case_count);
            let deadline: BlockNumberFor<T> =
                submitted_at.saturating_add(T::ChallengeWindow::get());
            CaseStatusOf::<T>::insert(case_id, CaseStatus::Open);
            CaseDeadline::<T>::insert(case_id, deadline);
            let _ = CasesByDeadline::<T>::try_append(deadline, case_id);
            Self::deposit_event(Event::CaseSubmitted {
                id: case_id,
                who: submitter.clone(),
            });
            Ok(().into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(2, 2))]
        #[allow(clippy::useless_conversion)]
        pub fn support_case(origin: OriginFor<T>, case_id: u128) -> DispatchResultWithPostInfo {
            let supporter = ensure_signed(origin)?;
            let supporters = CaseSupporters::<T>::get(case_id);
            if supporters.contains(&supporter) {
                return Err(Error::<T>::AlreadySupporting.into());
            }
            let _ = CaseSupporters::<T>::try_append(case_id, supporter.clone());
            Self::deposit_event(Event::CaseSupported {
                id: case_id,
                who: supporter.clone(),
            });
            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(2, 2))]
        #[allow(clippy::useless_conversion)]
        pub fn challenge_case(origin: OriginFor<T>, case_id: u128) -> DispatchResultWithPostInfo {
            let challenger = ensure_signed(origin)?;
            let challengers = CaseChallengers::<T>::get(case_id);
            if challengers.contains(&challenger) {
                return Err(Error::<T>::AlreadyChallenging.into());
            }
            let _ = CaseChallengers::<T>::try_append(case_id, challenger.clone());
            Self::deposit_event(Event::CaseChallenged {
                id: case_id,
                who: challenger.clone(),
            });
            Ok(().into())
        }
    }
}

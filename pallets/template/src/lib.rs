#![cfg_attr(not(feature = "std"), no_std)]

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
// #[cfg(test)]
mod impls;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
mod mock;
#[cfg(test)]
mod tests;

// Re-export pallet items so that they can be accessed from the crate namespace.
// use frame::prelude::*;
pub use pallet::*;

// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
    // Import various useful types required by all FRAME pallets.
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::fungible::{Inspect, Mutate},
        Blake2_128Concat,
    };
    use frame_system::pallet_prelude::*;

    pub type BalanceOf<T> =
        <<T as Config>::NativeCurrency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    // The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
    // (`Call`s) in this pallet.
    #[pallet::pallet]
    // pub struct Pallet<T>(_);
    pub struct Pallet<T>(core::marker::PhantomData<T>); // ???

    /// The pallet's configuration trait.
    ///
    /// All our types and constants a pallet depends on must be declared here.
    /// These types are defined generically and made concrete when the pallet is declared in the
    /// `runtime/src/lib.rs` file of your chain.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        type NativeCurrency: Inspect<Self::AccountId> + Mutate<Self::AccountId>;
    }

    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Kitty<T: Config> {
        pub dna: [u8; 32],
        pub owner: T::AccountId,
        pub price: Option<BalanceOf<T>>,
    }
    /// A storage item for this pallet.
    ///
    /// In this template, we are declaring a storage item called `Something` that stores a single
    /// `u32` value. Learn more about runtime storage here: <https://docs.substrate.io/build/runtime-storage/>
    #[pallet::storage]
    pub type Something<T> = StorageValue<_, u32>;

    #[pallet::storage]
    pub(super) type CountForKitties<T: Config> = StorageValue<Value = u32, QueryKind = ValueQuery>;

    #[pallet::storage]
    pub(super) type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], Kitty<T>>;

    #[pallet::storage]
    pub(super) type KittiesOwned<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<[u8; 32], ConstU32<100>>,
        ValueQuery,
    >;

    /// Events that functions in this pallet can emit.
    ///
    /// Events are a simple means of indicating to the outside world (such as dApps, chain explorers
    /// or other users) that some notable update in the runtime has occurred. In a FRAME pallet, the
    /// documentation for each event field and its parameters is added to a node's metadata so it
    /// can be used by external interfaces or tools.
    ///
    ///	The `generate_deposit` macro generates a function on `Pallet` called `deposit_event` which
    /// will convert the event type of your pallet into `RuntimeEvent` (declared in the pallet's
    /// [`Config`] trait) and deposit it using [`frame_system::Pallet::deposit_event`].
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A user has successfully set a new value.
        SomethingStored {
            /// The new value set.
            something: u32,
            /// The account who set the new value.
            who: T::AccountId,
        },
        Created {
            owner: T::AccountId,
            kitty_id: [u8; 32],
        },
        Transferred {
            from: T::AccountId,
            to: T::AccountId,
            kitty_id: [u8; 32],
        },
        PriceSet {
            owner: T::AccountId,
            kitty_id: [u8; 32],
            new_price: Option<BalanceOf<T>>,
        },
        Sold {
            buyer: T::AccountId,
            kitty_id: [u8; 32],
            price: BalanceOf<T>,
        },
    }

    /// Errors that can be returned by this pallet.
    ///
    /// Errors tell users that something went wrong so it's important that their naming is
    /// informative. Similar to events, error documentation is added to a node's metadata so it's
    /// equally important that they have helpful documentation associated with them.
    ///
    /// This type of runtime error can be up to 4 bytes in size should you want to return additional
    /// information.
    #[pallet::error]
    pub enum Error<T> {
        /// The value retrieved was `None` as no value was previously set.
        NoneValue,
        /// There was an attempt to increment the value in storage over `u32::MAX`.
        StorageOverflow,
        TooManyKitties,
        DuplicatedKitty,
        NoKitty,
        TooManyOwned,
        NotOwner,
        TransferToSelf,
        NotForSale,
        MaxPriceTooLow,
    }

    /// The pallet's dispatchable functions ([`Call`]s).
    ///
    /// Dispatchable functions allows users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// They must always return a `DispatchResult` and be annotated with a weight and call index.
    ///
    /// The [`call_index`] macro is used to explicitly
    /// define an index for calls in the [`Call`] enum. This is useful for pallets that may
    /// introduce new dispatchables over time. If the order of a dispatchable changes, its index
    /// will also change which will break backwards compatibility.
    ///
    /// The [`weight`] macro is used to assign a weight to each call.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a single u32 value as a parameter, writes the value
        /// to storage and emits an event.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::create_kitty())]
        pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let dna = Self::gen_dna();
            Self::mint(who, dna)?;
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::transfer())]
        pub fn transfer(
            origin: OriginFor<T>,
            to: T::AccountId,
            kitty_id: [u8; 32],
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;
            Self::do_transfer(from, to, kitty_id)?;
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_price())]
        pub fn set_price(
            origin: OriginFor<T>,
            kitty_id: [u8; 32],
            price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;

            Self::do_set_price(from, kitty_id, price)?;
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::buy_kitty())]
        pub fn buy_kitty(
            origin: OriginFor<T>,
            kitty_id: [u8; 32],
            max_price: BalanceOf<T>,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;
            Self::do_buy_kitty(from, kitty_id, max_price)?;
            Ok(())
        }
    }
}

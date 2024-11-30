#![cfg(test)]

use crate::*;
use crate::{self as pallet_kitties};
use frame::deps::frame_support::runtime;
use frame::deps::sp_io;
use frame::runtime::prelude::*;
use frame::testing_prelude::*;

type Balance = u64;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

#[runtime]
mod runtime {
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeTask,
        RuntimeFreezeReason,
        RuntimeHoldReason
    )]
    #[runtime::runtime]
    /// The "test runtime" that represents the state transition function for our blockchain.
    ///
    /// The runtime is composed of individual modules called "pallets", which you find see below.
    /// Each pallet has its own logic and storage, all of which can be combined together.
    pub struct TestRuntime;

    /// System: Mandatory system pallet that should always be included in a FRAME runtime.
    #[runtime::pallet_index(0)]
    pub type System = frame_system::Pallet<Runtime>;

    /// PalletBalances: Manages your blockchain's native currency. (i.e. DOT on Polkadot)
    #[runtime::pallet_index(1)]
    pub type PalletBalances = pallet_balances::Pallet<Runtime>;

    /// PalletKitties: The pallet you are building in this tutorial!
    #[runtime::pallet_index(2)]
    pub type PalletKitties = pallet_kitties::Pallet<Runtime>;
}

// Normally `System` would have many more configurations, but you can see that we use some macro
// magic to automatically configure most of the pallet for a "default test configuration".
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for TestRuntime {
    type Block = Block;
    type AccountData = pallet_balances::AccountData<Balance>;
}

// Normally `pallet_balances` would have many more configurations, but you can see that we use some
// macro magic to automatically configure most of the pallet for a "default test configuration".
#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for TestRuntime {
    type AccountStore = System;
    type Balance = Balance;
}

// This is the configuration of our Pallet! If you make changes to the pallet's `trait Config`, you
// will also need to update this configuration to represent that.
impl pallet_kitties::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = (); // TODO: investigate why this works
}

// We need to run most of our tests using this function: `new_test_ext().execute_with(|| { ... });`
// It simulates the blockchain database backend for our tests.
// If you forget to include this and try to access your Pallet storage, you will get an error like:
// "`get_version_1` called outside of an Externalities-provided environment."
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<TestRuntime>::default()
        .build_storage()
        .unwrap()
        .into()
}

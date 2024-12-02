#![cfg(test)]

use crate as pallet_kitties;
use frame_support::{construct_runtime, derive_impl};
use sp_runtime::BuildStorage;

type Balance = u64;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

construct_runtime! {
    pub struct TestRuntime {
        System: frame_system,
        PalletBalances: pallet_balances,
        PalletKitties: pallet_kitties,
    }
}
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for TestRuntime {
    type Block = Block;
    type AccountData = pallet_balances::AccountData<Balance>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for TestRuntime {
    type AccountStore = System;
    type Balance = Balance;
}

impl pallet_kitties::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type NativeCurrency = PalletBalances;
    type WeightInfo = ();
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

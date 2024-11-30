// Tests for the Kitties Pallet.
//
// Normally this file would be split into two parts: `mock.rs` and `tests.rs`.
// The `mock.rs` file would contain all the setup code for our `TestRuntime`.
// Then `tests.rs` would only have the tests for our pallet.
// However, to minimize the project, these have been combined into this single file.
//
// Learn more about creating tests for Pallets:
// https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html

// This flag tells rust to only run this file when running `cargo test`.
#![cfg(test)]

use crate::mock::{new_test_ext, RuntimeEvent, RuntimeOrigin, TestRuntime};
use crate::*;
// use frame::deps::frame_support::runtime;
// use frame::deps::sp_io;
// use frame::runtime::prelude::*;
use frame::testing_prelude::*;
use mock::{PalletBalances, PalletKitties, System};
// use frame::traits::fungible::*;

type Balance = u64;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

// In our "test runtime", we represent a user `AccountId` with a `u64`.
// This is just a simplification so that we don't need to generate a bunch of proper cryptographic
// public keys when writing tests. It is just easier to say "user 1 transfers to user 2".
// We create the constants `ALICE` and `BOB` to make it clear when we are representing users below.
const ALICE: u64 = 1;
const BOB: u64 = 2;

const DEFAULT_KITTY: Kitty<TestRuntime> = Kitty {
    dna: [0u8; 32],
    owner: 0,
    price: None,
};

#[test]
fn starting_template_is_sane() {
    new_test_ext().execute_with(|| {
        let event = Event::<TestRuntime>::Created {
            owner: ALICE,
            kitty_id: DEFAULT_KITTY.dna,
        };
        let _runtime_event: RuntimeEvent = event.into();
        let _call = Call::<TestRuntime>::create_kitty {};
        let result = PalletKitties::create_kitty(RuntimeOrigin::signed(BOB));
        assert_ok!(result);
    });
}

#[test]
fn system_and_balances_work() {
    // This test will just sanity check that we can access `System` and `PalletBalances`.
    new_test_ext().execute_with(|| {
        // We often need to set `System` to block 1 so that we can see events.
        System::set_block_number(1);
        // We often need to add some balance to a user to test features which needs tokens.
        assert_ok!(PalletBalances::force_set_balance(
            RuntimeOrigin::root(),
            ALICE,
            100
        ));
        assert_ok!(PalletBalances::force_set_balance(
            RuntimeOrigin::root(),
            BOB,
            100
        ));
        // assert_ok!(PalletBalances::mint_into(&ALICE, 100));
        // assert_ok!(PalletBalances::mint_into(&BOB, 100));
    });
}

#[test]
fn create_kitty_checks_signed() {
    new_test_ext().execute_with(|| {
        // Function returns OK if create_kitty is called/signed by a valid AccountID
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));

        // functions fails if called from unsigned msg
        assert_noop!(
            PalletKitties::create_kitty(RuntimeOrigin::none()),
            DispatchError::BadOrigin
        );
    })
}

#[test]
fn create_kitty_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        // Execute call
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));

        // fetch last event:
        let last_event = System::events().pop().expect("Event expected").event;
        match last_event {
            RuntimeEvent::PalletKitties(crate::Event::Created { owner, .. }) => {
                assert_eq!(owner, ALICE);
                // assert_eq!(kitty_id, DEFAULT_KITTY.dna); Ignore since "random"
            }
            _ => panic!("unexpected event"),
        }

        // Assert last event is the Created one
        // System::assert_last_event(
        //     Event::<TestRuntime>::Created {
        //         owner: 1,
        //         kitty_id: DEFAULT_KITTY.dna, // Check if this is correct (or generally created)
        //     }
        //     .into(),
        // );
    })
}

#[test]
fn count_for_kitties_created_correctly() {
    new_test_ext().execute_with(|| {
        assert_eq!(CountForKitties::<TestRuntime>::get(), u32::default());

        // A bit awkward... what is the difference between these 2?
        CountForKitties::<TestRuntime>::set(1337u32);
        assert_eq!(CountForKitties::<TestRuntime>::get(), 1337u32);
        CountForKitties::<TestRuntime>::put(1336u32);

        assert_ne!(CountForKitties::<TestRuntime>::get(), 1337u32);
        assert_eq!(CountForKitties::<TestRuntime>::get(), 1336u32);
    });
}

#[test]
fn mint_increment_counter() {
    new_test_ext().execute_with(|| {
        assert_eq!(CountForKitties::<TestRuntime>::get(), u32::default());

        // Check if necessary
        System::set_block_number(1);

        // Execute call
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));

        assert_eq!(CountForKitties::<TestRuntime>::get(), 1);
    })
}

#[test]
fn mint_error_on_overflow() {
    new_test_ext().execute_with(|| {
        CountForKitties::<TestRuntime>::put(u32::MAX);

        assert_noop!(
            PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)),
            Error::<TestRuntime>::TooManyKitties
        );
    });
}

// copied
#[test]
fn kitties_map_created_correctly() {
    new_test_ext().execute_with(|| {
        let zero_key = [0u8; 32];
        assert!(!Kitties::<TestRuntime>::contains_key(zero_key));
        Kitties::<TestRuntime>::insert(zero_key, DEFAULT_KITTY);
        assert!(Kitties::<TestRuntime>::contains_key(zero_key));
    })
}

//copied
#[test]
fn create_kitty_adds_to_map() {
    new_test_ext().execute_with(|| {
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));
        assert_eq!(Kitties::<TestRuntime>::iter().count(), 1);
    })
}

#[test]
fn raise_error_on_duplicated() {
    new_test_ext().execute_with(|| {
        assert_ok!(PalletKitties::mint(ALICE, [0u8; 32]));
        assert_eq!(Kitties::<TestRuntime>::iter().count(), 1);

        assert_noop!(
            PalletKitties::mint(BOB, [0u8; 32]),
            Error::<TestRuntime>::DuplicatedKitty
        );
        assert_eq!(Kitties::<TestRuntime>::iter().count(), 1);
    })
}

//copy
#[test]
fn kitty_struct_has_expected_traits() {
    new_test_ext().execute_with(|| {
        let kitty = DEFAULT_KITTY;
        let bytes = kitty.encode();
        let _decoded_kitty = Kitty::<TestRuntime>::decode(&mut &bytes[..]).unwrap();
        assert!(Kitty::<TestRuntime>::max_encoded_len() > 0);
        let _info = Kitty::<TestRuntime>::type_info();
    })
}

#[test]
fn mint_stores_owner_in_kitty() {
    new_test_ext().execute_with(|| {
        assert_ok!(PalletKitties::mint(1337, [42u8; 32]));
        let kitty = Kitties::<TestRuntime>::get([42u8; 32]).unwrap();
        assert_eq!(kitty.owner, 1337);
        assert_eq!(kitty.dna, [42u8; 32]);
    })
}

#[test]
fn create_kitty_unique() {
    new_test_ext().execute_with(|| {
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(BOB)));

        assert_eq!(CountForKitties::<TestRuntime>::get(), 2);
        assert_eq!(Kitties::<TestRuntime>::iter().count(), 2);
    })
}

#[test]
fn kitties_owned_creation() {
    new_test_ext().execute_with(|| {
        // Initially users have no kitties owned.
        assert_eq!(KittiesOwned::<TestRuntime>::get(ALICE).len(), 0);
        // Let's create two kitties.
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));
        // Now they should have two kitties owned.
        assert_eq!(KittiesOwned::<TestRuntime>::get(ALICE).len(), 2);
    })
}

//copy
#[test]
fn cannot_own_too_many_kitties() {
    new_test_ext().execute_with(|| {
        // If your max owned is different than 100, you will need to update this.
        for _ in 0..100 {
            assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));
        }
        assert_noop!(
            PalletKitties::create_kitty(RuntimeOrigin::signed(1)),
            Error::<TestRuntime>::TooManyOwned
        );
    });
}

#[test]
fn transfer_emits_event() {
    new_test_ext().execute_with(|| {
        // We need to set block number to 1 to view events.
        System::set_block_number(1);
        // Create a kitty to transfer
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));
        // Get the kitty id.
        let kitty_id = Kitties::<TestRuntime>::iter_keys().collect::<Vec<_>>()[0];
        assert_ok!(PalletKitties::transfer(
            RuntimeOrigin::signed(ALICE),
            BOB,
            kitty_id
        ));
        System::assert_last_event(
            Event::<TestRuntime>::Transferred {
                from: ALICE,
                to: BOB,
                kitty_id,
            }
            .into(),
        );
    });
}

#[test]
fn transfer_logic_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));
        // Starting state looks good.
        let kitty = &Kitties::<TestRuntime>::iter_values().collect::<Vec<_>>()[0];
        let kitty_id = kitty.dna;
        assert_eq!(kitty.owner, ALICE);
        assert_eq!(KittiesOwned::<TestRuntime>::get(ALICE), vec![kitty_id]);
        assert_eq!(KittiesOwned::<TestRuntime>::get(BOB), vec![]);
        // Cannot transfer to yourself.
        assert_noop!(
            PalletKitties::transfer(RuntimeOrigin::signed(ALICE), ALICE, kitty_id),
            Error::<TestRuntime>::TransferToSelf
        );
        // Cannot transfer a non-existent kitty.
        assert_noop!(
            PalletKitties::transfer(RuntimeOrigin::signed(ALICE), BOB, [0u8; 32]),
            Error::<TestRuntime>::NoKitty
        );
        // Cannot transfer kitty you do not own.
        assert_noop!(
            PalletKitties::transfer(RuntimeOrigin::signed(BOB), ALICE, kitty_id),
            Error::<TestRuntime>::NotOwner
        );
        // Transfer should work when parameters are right.
        assert_ok!(PalletKitties::transfer(
            RuntimeOrigin::signed(ALICE),
            BOB,
            kitty_id
        ));
        // Storage is updated correctly.
        assert_eq!(KittiesOwned::<TestRuntime>::get(ALICE), vec![]);
        assert_eq!(KittiesOwned::<TestRuntime>::get(BOB), vec![kitty_id]);
        let kitty = &Kitties::<TestRuntime>::iter_values().collect::<Vec<_>>()[0];
        assert_eq!(kitty.owner, BOB);
    });
}

// #[test]
// fn native_balance_associated_type_works() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(<<TestRuntime as Config>::PalletBalances as Mutate<_>>::mint_into(&ALICE, 1337));
//         assert_eq!(
//             <<TestRuntime as Config>::NativeBalance as Inspect<_>>::total_balance(&ALICE),
//             1337
//         );
//     });
// }

// #[test]
// fn balance_of_type_works() {
//     // Inside our tests, the `BalanceOf` type has a concrete type of `u64`.
//     let _example_balance: BalanceOf<TestRuntime> = 1337u64;
// }

#[test]
fn set_price_emits_event() {
    new_test_ext().execute_with(|| {
        // We need to set block number to 1 to view events.
        System::set_block_number(1);
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));
        let kitty_id = Kitties::<TestRuntime>::iter_keys().collect::<Vec<_>>()[0];
        assert_ok!(PalletKitties::set_price(
            RuntimeOrigin::signed(ALICE),
            kitty_id,
            Some(1337)
        ));
        // Assert the last event is `PriceSet` event with the correct information.
        System::assert_last_event(
            Event::<TestRuntime>::PriceSet {
                owner: ALICE,
                kitty_id,
                new_price: Some(1337),
            }
            .into(),
        );
    })
}

#[test]
fn set_price_logic_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));
        let kitty = &Kitties::<TestRuntime>::iter_values().collect::<Vec<_>>()[0];
        assert_eq!(kitty.price, None);
        let kitty_id = kitty.dna;
        assert_ok!(PalletKitties::set_price(
            RuntimeOrigin::signed(ALICE),
            kitty_id,
            Some(1337)
        ));
        let kitty = Kitties::<TestRuntime>::get(kitty_id).unwrap();
        assert_eq!(kitty.price, Some(1337));
    })
}
#[test]
fn do_buy_kitty_emits_event() {
    new_test_ext().execute_with(|| {
        // We need to set block number to 1 to view events.
        System::set_block_number(1);
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));
        let kitty_id = Kitties::<TestRuntime>::iter_keys().collect::<Vec<_>>()[0];
        assert_ok!(PalletKitties::set_price(
            RuntimeOrigin::signed(ALICE),
            kitty_id,
            Some(1337)
        ));
        // assert_ok!(PalletBalances::mint_into(&BOB, 100_000));

        assert_ok!(PalletBalances::force_set_balance(
            RuntimeOrigin::root(),
            BOB,
            100_000
        ));
        assert_ok!(PalletKitties::buy_kitty(
            RuntimeOrigin::signed(BOB),
            kitty_id,
            1337
        ));
        // Assert the last event by our blockchain is the `Created` event with the correct owner.
        System::assert_last_event(
            Event::<TestRuntime>::Sold {
                buyer: BOB,
                kitty_id,
                price: 1337,
            }
            .into(),
        );
    })
}

#[test]
fn do_buy_kitty_logic_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(PalletKitties::create_kitty(RuntimeOrigin::signed(ALICE)));
        let kitty = &Kitties::<TestRuntime>::iter_values().collect::<Vec<_>>()[0];
        let kitty_id = kitty.dna;
        assert_eq!(kitty.owner, ALICE);
        assert_eq!(KittiesOwned::<TestRuntime>::get(ALICE), vec![kitty_id]);
        // Cannot buy kitty which does not exist.
        assert_noop!(
            PalletKitties::buy_kitty(RuntimeOrigin::signed(BOB), [0u8; 32], 1337),
            Error::<TestRuntime>::NoKitty
        );
        // Cannot buy kitty which is not for sale.
        assert_noop!(
            PalletKitties::buy_kitty(RuntimeOrigin::signed(BOB), kitty_id, 1337),
            Error::<TestRuntime>::NotForSale
        );
        assert_ok!(PalletKitties::set_price(
            RuntimeOrigin::signed(ALICE),
            kitty_id,
            Some(1337)
        ));
        // Cannot buy kitty for a lower price.
        assert_noop!(
            PalletKitties::buy_kitty(RuntimeOrigin::signed(BOB), kitty_id, 1336),
            Error::<TestRuntime>::MaxPriceTooLow
        );
        // Cannot buy kitty if you don't have the funds.
        assert_noop!(
            PalletKitties::buy_kitty(RuntimeOrigin::signed(BOB), kitty_id, 1337),
            frame::arithmetic::ArithmeticError::Underflow
        );
        // Cannot buy kitty if it would kill your account (i.e. set your balance to 0).
        // assert_ok!(PalletBalances::mint_into(&BOB, 1337));

        assert_ok!(PalletBalances::force_set_balance(
            RuntimeOrigin::root(),
            BOB,
            1337
        ));
        assert!(
            PalletKitties::buy_kitty(RuntimeOrigin::signed(BOB), kitty_id, 1337).is_err(),
            // TODO: assert_noop on DispatchError::Token(TokenError::NotExpendable)
        );
        // When everything is right, it works.
        // assert_ok!(PalletBalances::mint_into(&BOB, 100_000));
        assert_ok!(PalletBalances::force_set_balance(
            RuntimeOrigin::root(),
            BOB,
            101_337
        ));
        assert_ok!(PalletKitties::buy_kitty(
            RuntimeOrigin::signed(BOB),
            kitty_id,
            1337
        ));
        // State is updated correctly.
        assert_eq!(KittiesOwned::<TestRuntime>::get(BOB), vec![kitty_id]);
        let kitty = Kitties::<TestRuntime>::get(kitty_id).unwrap();
        assert_eq!(kitty.owner, BOB);
        // Price is reset to `None`.
        assert_eq!(kitty.price, None);
        // BOB transferred funds to ALICE.
        assert_eq!(PalletBalances::free_balance(&ALICE), 1337);
        assert_eq!(PalletBalances::free_balance(&BOB), 100_000);
    })
}

//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Collectables;
use frame_benchmarking::v2::*;
use frame_support::traits::fungible::Inspect;
use frame_support::traits::fungible::Mutate;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_kitty() {
        let caller: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        create_kitty(RawOrigin::Signed(caller.clone()));

        let count = CountForKitties::<T>::get();
        assert_eq!(count, 1);

        let owned = KittiesOwned::<T>::get(caller);
        assert_eq!(owned.len(), 1);
    }

    #[benchmark]
    fn transfer() {
        let caller: T::AccountId = whitelisted_caller();
        let recipient: T::AccountId = account("bob", 0, 0);

        Pallet::<T>::create_kitty(RawOrigin::Signed(caller.clone()).into()).unwrap();
        let kitty_id = KittiesOwned::<T>::get(caller.clone())[0];

        #[extrinsic_call]
        transfer(
            RawOrigin::Signed(caller.clone()),
            recipient.clone(),
            kitty_id,
        );

        let recipient_owned = KittiesOwned::<T>::get(recipient.clone());
        assert_eq!(recipient_owned.len(), 1);
        assert_eq!(recipient_owned[0], kitty_id);

        let caller_owned = KittiesOwned::<T>::get(caller.clone());
        assert_eq!(caller_owned.len(), 0);
    }

    #[benchmark]
    fn set_price() {
        let caller: T::AccountId = whitelisted_caller();
        let price: BalanceOf<T> = 100u32.into();

        Pallet::<T>::create_kitty(RawOrigin::Signed(caller.clone()).into()).unwrap();
        let kitty_id = KittiesOwned::<T>::get(caller.clone())[0];
        assert_eq!(Kitties::<T>::get(kitty_id).unwrap().price, None);

        #[extrinsic_call]
        set_price(RawOrigin::Signed(caller.clone()), kitty_id, Some(price));
        assert_eq!(Kitties::<T>::get(kitty_id).unwrap().price, Some(price));
    }

    #[benchmark]
    fn buy_kitty() -> Result<(), BenchmarkError> {
        let seller: T::AccountId = whitelisted_caller();
        let buyer: T::AccountId = account("bob", 0, 0);

        let ed = T::NativeCurrency::minimum_balance();
        let price: BalanceOf<T> = 100u32.into();
        let balance: BalanceOf<T> = ed + price * 2u32.into();

        T::NativeCurrency::mint_into(&buyer, balance)?;
        T::NativeCurrency::mint_into(&seller, ed)?;

        Pallet::<T>::create_kitty(RawOrigin::Signed(seller.clone()).into())?;
        let kitty_id = KittiesOwned::<T>::get(seller.clone())[0];
        Pallet::<T>::set_price(
            RawOrigin::Signed(seller.clone()).into(),
            kitty_id,
            Some(price),
        )?;

        #[extrinsic_call]
        buy_kitty(RawOrigin::Signed(buyer.clone()), kitty_id, price);

        let kitty = Kitties::<T>::get(kitty_id).unwrap();
        assert_eq!(kitty.owner, buyer);
        assert_eq!(kitty.price, None);

        Ok(())
    }

    impl_benchmark_test_suite!(
        Template,
        crate::tests::new_test_ext(),
        crate::tests::TestRuntime
    );
}

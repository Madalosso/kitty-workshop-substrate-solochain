use super::*;
use codec::Encode;
use frame_support::pallet_prelude::*;
use frame_support::traits::fungible::Mutate;
use frame_support::traits::tokens::Preservation;
use sp_io::hashing::blake2_256;

impl<T: Config> Pallet<T> {
    pub fn gen_dna() -> [u8; 32] {
        let unique_payload = (
            frame_system::Pallet::<T>::parent_hash(),
            frame_system::Pallet::<T>::block_number(),
            frame_system::Pallet::<T>::extrinsic_index(),
            CountForKitties::<T>::get(),
        );
        let serialized_payload = unique_payload.encode();
        blake2_256(&serialized_payload)
    }

    pub fn do_transfer(from: T::AccountId, to: T::AccountId, dna: [u8; 32]) -> DispatchResult {
        // Not transferring to self
        ensure!(!from.eq(&to), Error::<T>::TransferToSelf);
        // (Workshop implementation)
        // Could be checked using == sign

        // Kitty exists
        // ensure!(Kitties::<T>::contains_key(dna), Error::<T>::NoKitty);
        let mut kitty = match Kitties::<T>::get(dna) {
            Some(kitty) => kitty,
            None => return Err(Error::<T>::NoKitty.into()),
        };
        // (Workshop implementation)
        // let mut kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::NoKitty)?;
        // Conclusion: Seems cleaner

        // caller/from is the owner
        ensure!(kitty.owner.eq(&from), Error::<T>::NotOwner);
        // (Workshop implementation)
        // Could be checked using == sign

        // (Workshop implementation)
        // kitty.owner = to.clone();
        // Kitty.owner assigned here Doesn't seem like it matters

        // Add kitty to owned map
        // let mut to_owned = KittiesOwned::<T>::get(to.clone());
        // (Workshop implementation)
        let mut to_owned = KittiesOwned::<T>::get(&to);
        // conclusion can borrow the to instead of cloning it. Better. Adopt.
        // TODO: Review borrow content

        to_owned
            .try_push(dna)
            .map_err(|_| Error::<T>::TooManyOwned)?;
        // (Workshop implementation) Same

        // Valid alternative?
        // KittiesOwned::<T>::try_append(&to, dna).map_err(|_| Error::<T>::TooManyOwned)?;
        // If so, maybe this removes the necessity of
        // KittiesOwned::<T>::insert(to, to_owned);
        // later?

        // Remove kitty from from_owned map
        // let mut from_owned = KittiesOwned::<T>::try_get(from)?;
        let mut from_owned = KittiesOwned::<T>::get(&from);

        let index =
            from_owned
                .iter()
                .enumerate()
                .find_map(|(index, &item)| if item.eq(&dna) { Some(index) } else { None });

        let index = match index {
            Some(index) => index,
            None => return Err(Error::<T>::NotOwner.into()),
        };
        from_owned.swap_remove(index);
        // (Workshop implementation)
        // if let Some(ind) = from_owned.iter().position(|&id| id == dna) {
        // 	from_owned.swap_remove(ind);
        // } else {
        // 	return Err(Error::<T>::NoKitty.into());
        // }
        // Conclusion: Way cleaner. Adopt. Review Iter().position()

        kitty.owner = to.clone();
        // kitty.price = None;
        Kitties::<T>::insert(dna, kitty);
        KittiesOwned::<T>::insert(&to, to_owned);
        KittiesOwned::<T>::insert(&from, from_owned);

        Self::deposit_event(Event::<T>::Transferred {
            from,
            to,
            kitty_id: dna,
        });
        Ok(())
    }

    pub fn mint(owner: T::AccountId, dna: [u8; 32]) -> DispatchResult {
        let kitty = Kitty {
            dna,
            owner: owner.clone(),
            price: None,
        };

        // Ensure dna not present already
        // match Kitties::<T>::contains_key(dna) {
        // 	true => return Err(Error::<T>::DuplicatedKitty.into()),
        // 	false => {},
        // }
        // Macro instead:
        ensure!(
            !Kitties::<T>::contains_key(dna),
            Error::<T>::DuplicatedKitty
        );

        // All storage in blockchain is Option<T>, so lets use default zero if not set
        let current_count = CountForKitties::<T>::get();

        // Error didn't have to include .into() due to the "?" sign.
        let updated_count = current_count
            .checked_add(1)
            .ok_or(Error::<T>::TooManyKitties)?;
        CountForKitties::<T>::set(updated_count);
        Kitties::<T>::insert(dna, kitty);
        KittiesOwned::<T>::try_append(&owner, dna).map_err(|_| Error::<T>::TooManyOwned)?;

        Self::deposit_event(Event::<T>::Created {
            owner,
            kitty_id: dna,
        });
        Ok(())
    }

    pub fn do_set_price(
        from: T::AccountId,
        kitty_id: [u8; 32],
        price: Option<BalanceOf<T>>,
    ) -> DispatchResult {
        let mut kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::NoKitty)?;
        ensure!(kitty.owner == from, Error::<T>::NotOwner);

        kitty.price = price;

        Kitties::<T>::insert(kitty_id, kitty);

        Self::deposit_event(Event::<T>::PriceSet {
            owner: from,
            kitty_id,
            new_price: price,
        });
        return Ok(());
    }

    pub fn do_buy_kitty(
        buyer: T::AccountId,
        kitty_id: [u8; 32],
        max_price: BalanceOf<T>,
    ) -> DispatchResult {
        let buyer_address = buyer.clone();
        // Question: Really necessary to check the existence of kitty_id if calling do_transfer (which already do that?)
        let kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::NoKitty)?;

        // Assert is for sale and buyer max price covers the sale price
        let price = match kitty.price {
            Some(price) => {
                if price > max_price {
                    return Err(Error::<T>::MaxPriceTooLow.into());
                }
                price
            }
            None => return Err(Error::<T>::NotForSale.into()),
        };

        T::NativeCurrency::transfer(&buyer, &kitty.owner, price, Preservation::Preserve)?;

        // maybe refactor to accept &mut buyer? ownership move cause `buyer_address`
        Self::do_transfer(kitty.owner, buyer.clone(), kitty_id)?;

        // Call set price to remove the price.
        // Q: Worth to use this method instead of directly setting the price to None?
        Self::do_set_price(buyer, kitty_id, None)?;

        Self::deposit_event(Event::<T>::Sold {
            buyer: buyer_address,
            kitty_id,
            price,
        });

        return Ok(());
    }
}

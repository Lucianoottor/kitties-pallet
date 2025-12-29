use super::*;
use frame::prelude::*;
use frame::traits::{BlakeTwo256, Hash};


impl<T: Config> Pallet<T> {

	pub fn do_breeding(
		_owner: T::AccountId,
		_kitty_id_1: [u8; 32],
		_kitty_id_2: [u8; 32],
	) -> DispatchResult {
		let kitty_1 = Kitties::<T>::get(_kitty_id_1).ok_or(Error::<T>::NoKitty)?;
		let kitty_2 = Kitties::<T>::get(_kitty_id_2).ok_or(Error::<T>::NoKitty)?;

		ensure!(kitty_1.owner == _owner, Error::<T>::NotOwner);
		ensure!(kitty_2.owner == _owner, Error::<T>::NotOwner);

		let dna_1 = kitty_1.dna;
		let dna_2 = kitty_2.dna;

		let mut new_dna = [0u8; 32];

		let random_hash = T::Hashing::hash_of(&(
			frame_system::Pallet::<T>::parent_hash(),
			frame_system::Pallet::<T>::block_number(),
			frame_system::Pallet::<T>::extrinsic_index().unwrap_or(0),
		));
		let random_bytes = random_hash.as_ref();
		for i in 0..32 {
			new_dna[i] = (dna_1[i] & random_bytes[i]) | (dna_2[i] & !random_bytes[i]);
		}
		Self::mint(_owner, new_dna)?;
		Ok(())
	}

	pub fn gen_dna() -> [u8; 32] {

		// form the payload to create uniqueness
		let parent_hash = frame_system::Pallet::<T>::parent_hash();
		let block_number = frame_system::Pallet::<T>::block_number();
		let extrinsic_index = frame_system::Pallet::<T>::extrinsic_index().unwrap_or(0);
		let kitty_count = CountForKitties::<T>::get();
		let payload = (
			parent_hash,
			block_number,
			extrinsic_index,
			kitty_count,
		);
		let hash_of = BlakeTwo256::hash_of(&payload).into();
		hash_of 
	}
	pub fn mint(owner: T::AccountId, dna: [u8; 32]) -> DispatchResult {
		// handle panic! to avoid DDoS
		ensure!(!Kitties::<T>::contains_key(dna), Error::<T>::DuplicateKitty);
		let kitty = Kitty { dna, owner: owner.clone(), price: None };
		let current_count = CountForKitties::<T>::get();
		// rust operations are unsafe (cant handle overflow for example), so use checked add
		let new_count = current_count.checked_add(1).ok_or(Error::<T>::TooManyKitties)?;
		// using valuequery, so it doesnt return an option, but an integer instead
		CountForKitties::<T>::set(new_count);
		KittiesOwned::<T>::try_append(owner.clone(), dna).map_err(|_| Error::<T>::TooManyOwnedKitties)?;
		Kitties::<T>::insert(dna, kitty);
		Self::deposit_event(Event::<T>::Created { owner });
		Ok(())
	}

	pub fn do_transfer(from: T::AccountId, to:T::AccountId, kitty_id: [u8;32]) -> DispatchResult {
		// check if from to are different
		ensure!(from != to, Error::<T>::SelfTransfer);
		// check if kitty exists
		let mut kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::NoKitty)?;
		// check if from is owner
		ensure!(kitty.owner == from, Error::<T>::NotOwner);
		kitty.owner = to.clone();
		kitty.price = None;


		// create the lists to update the storage
		// get owned kitties of target
		let mut to_owned = KittiesOwned::<T>::get(&to);
		// try to add the kitty to his onwed list
		to_owned.try_push(kitty_id).map_err(|_| Error::<T>::TooManyOwnedKitties)?;
		// get kitties from the sender
		let mut from_owned = KittiesOwned::<T>::get(&from);
		// remove the kitty from his owned list, searching for the kitty id in his list
		if let Some(pos) = from_owned.iter().position(|&id| id == kitty_id) {
			// remove if found
			from_owned.swap_remove(pos);
		} else {
			// error if he doesnt have the cat
			return Err(Error::<T>::NoKitty.into());
		}

		Kitties::<T>::insert(kitty_id, kitty);
		// update storage with created lists
		KittiesOwned::<T>::insert(&to, to_owned);
		KittiesOwned::<T>::insert(&from, from_owned);

		Self::deposit_event(Event::<T>::Transferred { from, to, kitty_id });
		Ok(())
	}
	
	pub fn do_set_price(
		owner: T::AccountId,
		kitty_id: [u8;32],
		price: BalanceOf<T>,
	) -> DispatchResult {
		let mut kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::NoKitty)?;
		ensure!(kitty.owner == owner, Error::<T>::NotOwner);
		kitty.price = Some(price);
		Kitties::<T>::insert(kitty_id, kitty);
		Self::deposit_event(Event::<T>::PriceSet { owner, kitty_id, price });
		Ok(())
	}

	pub fn do_buy_kitty(
		buyer: T::AccountId,
		kitty_id: [u8;32],
		price: BalanceOf<T>
	) -> DispatchResult {

		let kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::NoKitty)?;
		let real_price = kitty.price.ok_or(Error::<T>::NotForSale)?;
		ensure!(price >= real_price, Error::<T>::PriceTooLow);

		T::NativeBalance::transfer(
			&buyer,
			&kitty.owner,
			real_price,
			Preservation::Preserve,
		)?;

		Self::do_transfer(kitty.owner, buyer.clone(), kitty_id)?;
		Self::deposit_event(Event::<T>::Sold { buyer, kitty_id, price: real_price });
		Ok(())
	}

	pub fn do_abandon_kitty(
		owner: T::AccountId,
		kitty_id: [u8;32]
	) -> DispatchResult {

		let kitty = Kitties::<T>::get(&kitty_id).ok_or(Error::<T>::NoKitty)?;
		ensure!(kitty.owner == owner, Error::<T>::NotOwner);

		let mut owned = KittiesOwned::<T>::get(&owner);
		if let Some(pos) = owned.iter().position(|&id| id == kitty_id) {
			owned.swap_remove(pos);
		} else {
			return Err(Error::<T>::NoKitty.into());
		}
		Kitties::<T>::remove(&kitty_id);
		KittiesOwned::<T>::insert(owner.clone(), owned);
		Self::deposit_event(Event::<T>::Abandoned { owner, kitty_id });
		Ok(())

	}
}
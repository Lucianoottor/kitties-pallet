#![cfg_attr(not(feature = "std"), no_std)]

mod impls;
mod tests;

use frame::prelude::*;
use frame::traits::fungible::{Mutate, Inspect};
use frame::traits::tokens::Preservation;
pub use pallet::*;

#[frame::pallet(dev_mode)]
pub mod pallet {
	use super::*;

	// creating a pallet that holds a generic "T" that represents the runtime
	#[pallet::pallet]
	pub struct Pallet<T>(core::marker::PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type NativeBalance: Mutate<Self::AccountId> + Inspect<Self::AccountId>;
	}
					
	pub type BalanceOf<T> =
		<<T as Config>::NativeBalance as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 32],
		pub owner: T::AccountId,
		pub price: Option<BalanceOf<T>>,
	}

	#[pallet::storage]
	// before Default type
	// pub(super) type CountForKitties<T: Config> = StorageValue<Value = u32>;
	pub(super) type CountForKitties<T: Config> = StorageValue<Value = u32, QueryKind = ValueQuery>;

	#[pallet::storage]
	pub(super) type Kitties<T: Config> = StorageMap<Key = [u8; 32], Value = Kitty<T>>;
	// owned items are redundant, but when we need to list owned kitties it is useful, mostly we are in a lose-lose situation developing blockchains
	// storage operations are expensive, need to think on the best way out of it
	#[pallet::storage]
	pub(super) type KittiesOwned<T: Config> = StorageMap<Key = T::AccountId, Value = BoundedVec::<[u8; 32], ConstU32<100>>, QueryKind = ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Created { owner: T::AccountId },
		Transferred { from: T::AccountId, to: T::AccountId, kitty_id: [u8; 32] },
		PriceSet { owner: T::AccountId, kitty_id: [u8; 32], price: BalanceOf<T> },
		Sold { buyer: T::AccountId, kitty_id: [u8; 32], price: BalanceOf<T> },
		Abandoned { owner: T::AccountId, kitty_id: [u8; 32] },
	}

	#[pallet::error]
    pub enum Error<T> {
        TooManyKitties,
		DuplicateKitty,
		TooManyOwnedKitties,
		NoKitty,
		SelfTransfer,
		NotOwner,
		NotForSale,
		PriceTooLow,
    }

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		pub fn create_kitty(
			origin: OriginFor<T>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let dna = Self::gen_dna();
			Self::mint(who, dna)?;
			Ok(())
		}

		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: [u8;32],
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			Self::do_transfer(from, to, kitty_id)?;
			Ok(())
		}

		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: [u8;32],
			price: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_set_price(who, kitty_id, price)?;
			Ok(())
		}

		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: [u8;32],
			price: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_buy_kitty(who, kitty_id, price)?;
			Ok(())
		}

		pub fn abandon_kitty(
			origin: OriginFor<T>,
			kitty_id: [u8;32]
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_abandon_kitty(who, kitty_id)?;
			Ok(())
		}

		pub fn breed_kitties(
			origin: OriginFor<T>,
			kitty_id_1: [u8;32],
			kitty_id_2: [u8;32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_breeding(who, kitty_id_1, kitty_id_2)?;
			Ok(())
		}
	}
}

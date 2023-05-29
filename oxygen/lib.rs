#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

pub use self::oxygen::OxygenRef;

#[openbrush::contract]
pub mod oxygen {

    use openbrush::contracts::ownable::*;
    use openbrush::contracts::psp22::extensions::mintable::*;
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Oxygen {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        ownable: ownable::Data,
    }

    impl PSP22 for Oxygen {}
    impl Ownable for Oxygen {}
    impl PSP22Mintable for Oxygen {}

    impl Oxygen {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let mut instance = Self::default();
            instance
                ._mint_to(instance.env().caller(), initial_supply)
                .expect("Should mint");
            instance._init_with_owner(instance.env().caller());
            instance
        }
    }
}

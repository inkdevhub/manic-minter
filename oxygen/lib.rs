#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

pub use self::oxygen::OxygenRef;

#[openbrush::contract]
pub mod oxygen {

    // imports from openbrush
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

    // Section contains default implementation without any modifications
    impl PSP22 for Oxygen {}
    impl Ownable for Oxygen {}
    impl PSP22Mintable for Oxygen {
        #[ink(message)]
        #[openbrush::modifiers(only_owner)]
        fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._mint_to(account, amount)
        }
    }

    impl Oxygen {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let mut _instance = Self::default();
            _instance
                ._mint_to(_instance.env().caller(), initial_supply)
                .expect("Should mint");
            _instance._init_with_owner(_instance.env().caller());
            _instance
        }
    }
}
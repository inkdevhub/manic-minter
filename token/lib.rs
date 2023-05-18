#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]
        
#[openbrush::contract]
pub mod my_psp22 {
    
    // imports from openbrush
	use openbrush::traits::Storage;
	use openbrush::contracts::ownable::*;
	use openbrush::contracts::psp22::extensions::mintable::*;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Token {
    	#[storage_field]
		psp22: psp22::Data,
		#[storage_field]
		ownable: ownable::Data,
    }
    
    // Section contains default implementation without any modifications
	impl PSP22 for Token {}
	impl Ownable for Token {}
	impl PSP22Mintable for Token {
		#[ink(message)]
		#[openbrush::modifiers(only_owner)]
		fn mint(
            &mut self,
            account: AccountId,
			amount: Balance
        ) -> Result<(), PSP22Error> {
			self._mint_to(account, amount)
		}
	}
     
    impl Token {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let mut _instance = Self::default();
			_instance._mint_to(_instance.env().caller(), initial_supply).expect("Should mint"); 
			_instance._init_with_owner(_instance.env().caller());
			_instance
        }
    }
}
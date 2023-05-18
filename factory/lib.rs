#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
mod factory {
    use crate::ensure;
    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        DefaultEnvironment,
    };
    #[ink(storage)]
    pub struct Factory {
        /// Contract owner
        owner: AccountId,
        /// Token contract address
        token_contract: AccountId,
        /// Minting price. Caller must pay this price to mint one new token from Token contract
        price: Balance,
    }
    
    /// The Factory error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// The call is not allowed if the caller is not the owner of the contract
        NotOwner,
        /// Returned if the token contract account is not set during the contract creation.
        ContractNotSet
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink::trait_definition]
    pub trait Minting {
        /// Mint new tokens from Token contract
        #[ink(message, payable)]
        fn mint(&mut self, amount: Balance) -> Result<()>;
    }

    impl Factory {
        #[ink(constructor)]
        pub fn new(contract_acc: AccountId) -> Self {
            Self {
                owner: Self::env().caller(),
                token_contract: contract_acc,
                price: 1,
            }
        }
    }

    impl Minting for Factory {
        #[ink(message, payable)]
        fn mint(&mut self, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            ensure!(
                self.token_contract != AccountId::from([0x0; 32]),
                Error::ContractNotSet
            );
            ensure!(
                self.price == self.env().transferred_value(),
                Error::InsufficientBalance
            );

            let _mint_result = build_call::<DefaultEnvironment>()
                .call(self.token_contract)
                .gas_limit(5000000000)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP34::mint")))
                        .push_arg(caller)
                        .push_arg(amount)
                )
                .returns::<()>()
                .try_invoke();
            ink::env::debug_println!("mint_result: {:?}", _mint_result);
            Ok(())
        }
    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        use super::*;

        /// Test error ContractNotSet.
        #[ink::test]
        fn contract_not_set_works() {
            let mut factory = Factory::new([0x0; 32].into());
            assert_eq!(
                factory.mint(50),
                Err(Error::ContractNotSet)
            );
        }
        
        /// Test error InsufficientBalance.
        #[ink::test]
        fn insufficient_balance_works() {
            let mut factory = Factory::new([0x1; 32].into());
            assert_eq!(
                factory.mint(50),
                Err(Error::InsufficientBalance)
            );
        }
    }

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = FactoryRef::default();

            // When
            let contract_account_id = client
                .instantiate("factory", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<FactoryRef>(contract_account_id.clone())
                .call(|factory| factory.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = FactoryRef::new(false);
            let contract_account_id = client
                .instantiate("factory", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<FactoryRef>(contract_account_id.clone())
                .call(|factory| factory.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<FactoryRef>(contract_account_id.clone())
                .call(|factory| factory.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<FactoryRef>(contract_account_id.clone())
                .call(|factory| factory.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr $(,)? ) => {{
        if !$x {
            return Err($y.into());
        }
    }};
}

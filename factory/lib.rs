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
        ContractNotSet,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink::trait_definition]
    pub trait Minting {
        /// Mint new tokens from Token contract
        #[ink(message, payable)]
        fn mint(&mut self, amount: Balance) -> Result<()>;

        #[ink(message)]
        fn set_price(&mut self, price: Balance) -> Result<()>;

        #[ink(message)]
        fn get_price(&self) -> Balance;
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
                        .push_arg(amount),
                )
                .returns::<()>()
                .try_invoke();
            ink::env::debug_println!("mint_result: {:?}", _mint_result);
            Ok(())
        }

        #[ink(message)]
        fn set_price(&mut self, price: Balance) -> Result<()> {
            ensure!(self.env().caller() == self.owner, Error::NotOwner);
            self.price = price;
            Ok(())
        }

        #[ink(message)]
        fn get_price(&self) -> Balance {
            self.price
        }
    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        /// Test error ContractNotSet.
        #[ink::test]
        fn contract_not_set_works() {
            let mut factory = Factory::new([0x0; 32].into());
            assert_eq!(factory.mint(50), Err(Error::ContractNotSet));
        }

        /// Test error InsufficientBalance.
        #[ink::test]
        fn insufficient_balance_works() {
            let mut factory = Factory::new([0x1; 32].into());
            assert_eq!(factory.mint(50), Err(Error::InsufficientBalance));
        }

        /// Test setting price
        #[ink::test]
        fn set_price_works() {
            let accounts = default_accounts();
            let mut factory = Factory::new([0x0; 32].into());
            assert!(factory.set_price(100).is_ok());
            assert_eq!(factory.get_price(), 100);

            // Non owner fails to set price
            set_sender(accounts.bob);
            assert_eq!(factory.set_price(100), Err(Error::NotOwner));
        }

        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<Environment>()
        }

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<Environment>(sender);
        }
    }

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use crate::factory::FactoryRef;
        use ink::primitives::AccountId;
        use ink_e2e::build_message;
        // use openbrush::contracts::psp22::psp22_external::PSP22;
        use my_psp22::my_psp22::TokenRef;
        use openbrush::contracts::ownable::ownable_external::Ownable;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        const AMOUNT: Balance = 100;

        fn get_alice_account_id() -> AccountId {
            let alice = ink_e2e::alice::<ink_e2e::PolkadotConfig>();
            let alice_account_id_32 = alice.account_id();
            let alice_account_id = AccountId::try_from(alice_account_id_32.as_ref()).unwrap();

            alice_account_id
        }

        #[ink_e2e::test(additional_contracts = "factory/Cargo.toml token/Cargo.toml")]
        async fn e2e_minting_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let initial_balance: Balance = 1_000_000;

            // Instantiate Token contract
            let token_constructor = TokenRef::new(initial_balance);
            let alice_account_id = get_alice_account_id();

            let token_account_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), token_constructor, 0, None)
                .await
                .expect("token instantiate failed")
                .account_id;

            // Instantiate factory contract
            let factory_constructor = FactoryRef::new(token_account_id);
            let factory_account_id = client
                .instantiate("factory", &ink_e2e::alice(), factory_constructor, 0, None)
                .await
                .expect("factory instantiate failed")
                .account_id;

            // Set Factory contract to be the owner of Token contract
            let change_owner = build_message::<TokenRef>(token_account_id.clone())
                .call(|p| p.transfer_ownership(factory_account_id));
            client
                .call(&ink_e2e::alice(), change_owner, 0, None)
                .await
                .expect("calling `transfer_ownership` failed");

            // Verify that Factory is the Token contract owner
            let owner = build_message::<TokenRef>(token_account_id.clone()).call(|p| p.owner());
            let owner_result = client
                .call_dry_run(&ink_e2e::alice(), &owner, 0, None)
                .await
                .return_value();
            assert_eq!(owner_result, factory_account_id);

            // Contract owner sets price
            let price_message = build_message::<FactoryRef>(factory_account_id.clone())
                .call(|factory| factory.set_price(100));
            client
                .call(&ink_e2e::alice(), price_message, 0, None)
                .await
                .expect("calling `set_price` failed");

            // Bob mints a token fails since no payment was made
            let mint_message = build_message::<FactoryRef>(factory_account_id.clone())
                .call(|factory| factory.mint(AMOUNT));
            let failed_mint_result = client
                .call_dry_run(&ink_e2e::bob(), &mint_message, 0, None)
                .await
                .return_value();
            assert_eq!(failed_mint_result, Err(Error::InsufficientBalance));

            // Bob mints a token
            client
                .call(&ink_e2e::bob(), mint_message, 100, None)
                .await
                .expect("calling `pink_mint` failed");

            // Check contract balance
            if let Ok(balance) = client.balance(factory_account_id).await {
                assert_eq!(balance, 100);
            }

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

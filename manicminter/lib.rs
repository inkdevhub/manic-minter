#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
mod manicminter {
    use crate::ensure;
    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        DefaultEnvironment,
    };
    #[ink(storage)]
    pub struct ManicMinter {
        /// Contract owner
        owner: AccountId,
        /// Token contract address
        token_contract: AccountId,
        /// Minting price. Caller must pay this price to mint one new token from Token contract
        price: Balance,
    }

    /// The ManicMinter error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        BadMintValue,
        /// The call is not allowed if the caller is not the owner of the contract
        NotOwner,
        /// Returned if the token contract account is not set during the contract creation.
        ContractNotSet,
        /// Returned if multiplication of price and amount overflows
        OverFlow,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink::trait_definition]
    pub trait Minting {
        /// Mint new tokens from Token contract
        #[ink(message, payable)]
        fn manic_mint(&mut self, amount: Balance) -> Result<()>;

        /// Set minting price for one Oxygen token
        #[ink(message)]
        fn set_price(&mut self, price: Balance) -> Result<()>;

        /// Get minting price for one Oxygen token
        #[ink(message)]
        fn get_price(&self) -> Balance;
    }

    impl ManicMinter {
        #[ink(constructor)]
        pub fn new(contract_acc: AccountId) -> Self {
            Self {
                owner: Self::env().caller(),
                token_contract: contract_acc,
                price: 0,
            }
        }
    }

    impl Minting for ManicMinter {
        #[ink(message, payable)]
        fn manic_mint(&mut self, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            ensure!(
                self.token_contract != AccountId::from([0x0; 32]),
                Error::ContractNotSet
            );
            if let Some(value) = (amount as u128).checked_mul(self.price) {
                let transferred_value = self.env().transferred_value();
                if transferred_value != value {
                    return Err(Error::BadMintValue);
                }
            }
            match (amount as u128).checked_mul(self.price) {
                Some(value) => {
                    let transferred_value = self.env().transferred_value();
                    if transferred_value != value {
                        return Err(Error::BadMintValue);
                    }
                }
                None => {
                    return Err(Error::OverFlow);
                }
            }

            let _mint_result = build_call::<DefaultEnvironment>()
                .call(self.token_contract)
                .gas_limit(5000000000)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP22Mintable::mint")))
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
            let mut manicminter = ManicMinter::new([0x0; 32].into());
            assert_eq!(manicminter.manic_mint(50), Err(Error::ContractNotSet));
        }

        /// Test setting price
        #[ink::test]
        fn set_price_works() {
            let accounts = default_accounts();
            let mut manicminter = ManicMinter::new([0x0; 32].into());
            assert!(manicminter.set_price(100).is_ok());
            assert_eq!(manicminter.get_price(), 100);

            // Non owner fails to set price
            set_sender(accounts.bob);
            assert_eq!(manicminter.set_price(100), Err(Error::NotOwner));
        }

        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<Environment>()
        }

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<Environment>(sender);
        }
    }

    /// ink! end-to-end (E2E) tests
    ///
    /// cargo test --features e2e-tests -- --nocapture
    ///
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use crate::manicminter::ManicMinterRef;
        use ink::primitives::AccountId;
        use ink_e2e::build_message;
        use my_psp22::my_psp22::TokenRef;
        use openbrush::contracts::ownable::ownable_external::Ownable;
        use openbrush::contracts::psp22::psp22_external::PSP22;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        const AMOUNT: Balance = 100;
        const PRICE: Balance = 10;

        /// Helper to get Bob's account_id from `ink_e2e::bob()` PairSigner
        fn get_bob_account_id() -> AccountId {
            let bob = ink_e2e::bob::<ink_e2e::PolkadotConfig>();
            let bob_account_id_32 = bob.account_id();
            let bob_account_id = AccountId::try_from(bob_account_id_32.as_ref()).unwrap();

            bob_account_id
        }

        #[ink_e2e::test(additional_contracts = "manicminter/Cargo.toml token/Cargo.toml")]
        async fn e2e_minting_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let initial_balance: Balance = 1_000_000;

            // Instantiate Token contract
            let token_constructor = TokenRef::new(initial_balance);

            let token_account_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), token_constructor, 0, None)
                .await
                .expect("token instantiate failed")
                .account_id;

            // Instantiate manic-minter contract
            let manic_minter_constructor = ManicMinterRef::new(token_account_id);
            let manic_minter_account_id = client
                .instantiate(
                    "manic-minter",
                    &ink_e2e::alice(),
                    manic_minter_constructor,
                    0,
                    None,
                )
                .await
                .expect("manic-minter instantiate failed")
                .account_id;

            // Set ManicMinter contract to be the owner of Token contract
            let change_owner = build_message::<TokenRef>(token_account_id.clone())
                .call(|p| p.transfer_ownership(manic_minter_account_id));
            client
                .call(&ink_e2e::alice(), change_owner, 0, None)
                .await
                .expect("calling `transfer_ownership` failed");

            // Verify that ManicMinter is the Token contract owner
            let owner = build_message::<TokenRef>(token_account_id.clone()).call(|p| p.owner());
            let owner_result = client
                .call_dry_run(&ink_e2e::alice(), &owner, 0, None)
                .await
                .return_value();
            assert_eq!(owner_result, manic_minter_account_id);

            // Contract owner sets price
            let price_message = build_message::<ManicMinterRef>(manic_minter_account_id.clone())
                .call(|manicminter| manicminter.set_price(PRICE));
            client
                .call(&ink_e2e::alice(), price_message, 0, None)
                .await
                .expect("calling `set_price` failed");

            // Bob mints a token fails since no payment was made
            let mint_message = build_message::<ManicMinterRef>(manic_minter_account_id.clone())
                .call(|manicminter| manicminter.manic_mint(AMOUNT));
            let failed_mint_result = client
                .call_dry_run(&ink_e2e::bob(), &mint_message, 0, None)
                .await
                .return_value();
            assert_eq!(failed_mint_result, Err(Error::BadMintValue));

            // Bob mints a token
            client
                .call(&ink_e2e::bob(), mint_message, PRICE * AMOUNT, None)
                .await
                .expect("calling `pink_mint` failed");

            // Verify that tokens were minted on Token contract
            let bob_account_id = get_bob_account_id();
            let balance_message = build_message::<TokenRef>(token_account_id.clone())
                .call(|p| p.balance_of(bob_account_id));
            let token_balance = client
                .call_dry_run(&ink_e2e::bob(), &balance_message, 0, None)
                .await
                .return_value();
            assert_eq!(token_balance, AMOUNT);

            // Check manic-minter contract balance
            if let Ok(balance) = client.balance(manic_minter_account_id).await {
                assert_eq!(balance, AMOUNT * PRICE);
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

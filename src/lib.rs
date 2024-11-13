use near_sdk::{
    AccountId, PanicOnDefault, borsh::{BorshDeserialize, BorshSerialize, self},
    env, json_types::U128, near_bindgen,
};
use near_sdk_contract_tools::{Owner, ft::*, owner::{*, hooks::OnlyOwner}};
use near_sdk_contract_tools::standard::nep148::ContractMetadata;


#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault, Owner, FungibleToken)]
#[fungible_token(mint_hook = "OnlyOwner")]
#[near_bindgen]
pub struct Contract {}

// #[near_bindgen]
impl Contract {
    // #[init]
    pub fn new() {
        env::log_str("new method");
        // let mut contract = Self {};

        // Owner::init(&mut contract, &env::predecessor_account_id());
        // Nep148Controller::set_metadata(
        //     &mut contract,
        //     &ContractMetadata::new("PUSH".to_string(), "PTB".to_string(), 5),
        // );

        // Nep141Controller::mint(
        //     &mut contract,
        //     &Nep141Mint {
        //         amount: 1000u128,
        //         receiver_id: std::borrow::Cow::Borrowed(&env::predecessor_account_id()),
        //         memo: None,
        //     },
        // )
        // .unwrap_or_else(|e| env::panic_str(&e.to_string()));

        // contract
    }

    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        Nep141Controller::mint(
            self,
            &Nep141Mint {
                amount: amount.into(),
                receiver_id: std::borrow::Cow::Borrowed(&account_id),
                memo: None,
            },
        )
        .unwrap_or_else(|e| env::panic_str(&e.to_string()));
    }

    pub fn burn(&mut self, amount: U128) {
        Nep141Controller::burn(
            self,
            &Nep141Burn {
                amount: amount.into(),
                owner_id: std::borrow::Cow::Borrowed(&env::predecessor_account_id()),
                memo: None,
            },
        )
        .unwrap_or_else(|e| env::panic_str(&e.to_string()));
    }
}

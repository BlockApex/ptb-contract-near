use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::{
    FungibleToken, FungibleTokenCore, FungibleTokenResolver,
};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, PromiseOrValue, require,
};

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    FungibleToken,
    Metadata,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner_id: AccountId,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        let owner_id = env::predecessor_account_id();
        let total_supply = U128(1000);
        let name = "PUSH".to_string();
        let symbol = "PTB".to_string();
        let decimals = 5;

        let mut token = FungibleToken::new(StorageKey::FungibleToken);
        token.internal_register_account(&owner_id);
        token.internal_deposit(&owner_id, total_supply.0);

        let metadata = FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name,
            symbol,
            icon: None,
            reference: None,
            reference_hash: None,
            decimals,
        };

        let metadata = LazyOption::new(StorageKey::Metadata, Some(&metadata));

        Self { token, metadata, owner_id }
    }

    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Only the owner can mint tokens"
        );
        self.token.internal_deposit(&account_id, amount.0);
    }

    pub fn burn(&mut self, amount: U128) {
        let account_id = env::predecessor_account_id();
        self.token.internal_withdraw(&account_id, amount.0);
    }
}

// #[near_bindgen]
// impl FungibleTokenCore for Contract {
//     // Implement required methods for FungibleTokenCore
// }

// #[near_bindgen]
// impl FungibleTokenResolver for Contract {
//     // Implement required methods for FungibleTokenResolver
// }

// #[near_bindgen]
// impl StorageManagement for Contract {
//     // Implement required methods for StorageManagement
// }

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

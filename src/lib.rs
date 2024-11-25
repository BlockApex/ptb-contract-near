use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::{
    FungibleToken, FungibleTokenCore, FungibleTokenResolver,
};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::borsh::BorshDeserialize;
use near_sdk::borsh::BorshSerialize;
use near_sdk::collections::LazyOption;
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{U128, U64};
use near_sdk::{
    env, log, near, require, AccountId, BorshStorageKey, NearToken, PanicOnDefault, PromiseOrValue,
};

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct EmissionsAccount {
    pub initial_emissions: U64,
    pub decay_factor: f64,
    pub current_month: u32,
    pub current_emissions: U64,
    pub last_mint_timestamp: U64,
}

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct RafflePool {
    pub pool_id: u32,
    pub amount: U128,
    pub total_amount: U128,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TappingPool {
    pub pool_id: u32,
    pub amount: U128,
}

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    emissions_account: LookupMap<AccountId, EmissionsAccount>,
    loot_raffle_pool: LookupMap<u32, RafflePool>,
    global_tapping_pool: LookupMap<u32, TappingPool>,
    owner_id: AccountId,
    proposed_owner: Option<AccountId>, //field for proposed owner
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "https://red-defensive-termite-556.mypinata.cloud/ipfs/QmUCUAABBsqkhSw3HoeMtecwVAeKBmxUgj2GLwmxuNojbV";

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    FungibleToken,
    Metadata,
}

/// Initializes the contract with the given total supply
#[near]
impl Contract {
    #[init]
    pub fn new_default_meta(total_supply: U128) -> Self {
        require!(!env::state_exists(), "Already initialized");
        let caller_id: AccountId = env::predecessor_account_id();

        let mut this = Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(
                StorageKey::Metadata,
                Some(&FungibleTokenMetadata {
                    spec: FT_METADATA_SPEC.to_string(),
                    name: "PUSH THE BUTTON PTB".to_string(),
                    symbol: "PUSH".to_string(),
                    icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                    reference: None,
                    reference_hash: None,
                    decimals: 5,
                }),
            ),
            emissions_account: LookupMap::new(b"e"),
            loot_raffle_pool: LookupMap::new(b"l"),
            global_tapping_pool: LookupMap::new(b"g"),
            owner_id: caller_id.clone(),
            proposed_owner: None,
        };
        // Initialize Emissions Account
        this.emissions_account.insert(
            &this.owner_id,
            &EmissionsAccount {
                initial_emissions: U64(3_000_000_000), 
               decay_factor: 0.8705505633,            
                current_month: 0,                      
                current_emissions: U64(3_000_000_000), 
               last_mint_timestamp: U64(env::block_timestamp()), // Wrap timestamp in U64
            },
        );

        // Initialize Raffle and Tapping Pools
        this.loot_raffle_pool.insert(
            &1,
            &RafflePool {
                pool_id: 1,                     
                amount: U128(50_000_000_00000), 
                total_amount: U128(0),          
            },
        );

        this.global_tapping_pool.insert(
            &2,
            &TappingPool {
                pool_id: 2,                        
                amount: U128(1_000_000_000_00000), 
            },
        );
        this.token.internal_register_account(&this.owner_id);
        this.token
            .internal_deposit(&this.owner_id, total_supply.into());

        this
    }

    /// Initiate Ownership Transfer
    #[payable]
    pub fn initiate_ownership_transfer(&mut self, new_owner: AccountId) {
        assert_one_yocto();
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Only the current owner can initiate an ownership transfer."
        );
        require!(
            new_owner != self.owner_id,
            "New owner cannot be the current owner."
        );

        self.proposed_owner = Some(new_owner.clone());
        log!("Ownership transfer initiated to: {}", new_owner);
    }

    #[payable]
    /// Accept Ownership Transfer
    pub fn accept_ownership(&mut self) {
        assert_one_yocto();
        let proposed_owner = self
            .proposed_owner
            .clone()
            .expect("No ownership transfer initiated.");
        require!(
            env::predecessor_account_id() == proposed_owner,
            "Only the proposed owner can accept the ownership transfer."
        );

        // Transfer ownership
        self.owner_id = proposed_owner.clone();
        self.proposed_owner = None;

        // Ensure the new owner has an emissions account
        if self.emissions_account.get(&self.owner_id).is_none() {
            self.emissions_account.insert(
                &self.owner_id,
                &EmissionsAccount {
                    initial_emissions: U64(3_000_000_000),
                    decay_factor: 0.8705505633,
                    current_month: 0,
                    current_emissions: U64(3_000_000_000), 
                    last_mint_timestamp: U64(env::block_timestamp()), 
                },
            );
            log!(
                "Initialized emissions account for new owner: {}",
                self.owner_id
            );
        }

        // Ensure the new owner has a loot raffle pool account
        if self.loot_raffle_pool.get(&1).is_none() {
            self.loot_raffle_pool.insert(
                &1,
                &RafflePool {
                    pool_id: 1,
                    amount: U128(50_000_000_00000), 
                    total_amount: U128(0),
                },
            );
            log!(
                "Initialized loot raffle pool for new owner: {}",
                self.owner_id
            );
        }

        // Ensure the new owner has a global tapping pool account
        if self.global_tapping_pool.get(&2).is_none() {
            self.global_tapping_pool.insert(
                &2,
                &TappingPool {
                    pool_id: 2,
                    amount: U128(1_000_000_000_00000), // Wrap integer in U128
                },
            );
            log!(
                "Initialized global tapping pool for new owner: {}",
                self.owner_id
            );
        }

        log!("Ownership successfully transferred to: {}", self.owner_id);
    }

    /// Check Current and Proposed Owners
    pub fn get_owners(&self) -> (AccountId, Option<AccountId>) {
        log!("Owner ID: {}", self.owner_id.clone());
        log!("Proposed Owner ID: {:?}", self.proposed_owner);
        (self.owner_id.clone(), self.proposed_owner.clone())
    }

    #[payable]
    pub fn mint(&mut self) {
        assert_one_yocto();
        require!(
            env::attached_deposit() == NearToken::from_yoctonear(1),
            "1 yoctoNEAR must be attached for this call"
        );

        let caller_id: AccountId = env::predecessor_account_id();
        log!("Caller ID: {}", caller_id);
        log!("Owner ID: {}", self.owner_id);

        require!(caller_id == self.owner_id, "Caller is not the owner");

        // Step 1: Retrieve emissions_account, loot_raffle_pool_account, and global_tapping_pool
        let mut emissions_account = self
            .emissions_account
            .get(&self.owner_id.clone())
            .expect("Emissions account not found");
        let mut loot_raffle_pool_account = self
            .loot_raffle_pool
            .get(&1)
            .expect("Loot raffle pool account not found");
        let mut global_tapping_pool = self
            .global_tapping_pool
            .get(&2)
            .expect("Global tapping pool not found");

        // Step 2: Get the current Unix timestamp
        let current_timestamp: u64 = env::block_timestamp() / 1_000_000_000; // Convert from nanoseconds to seconds

        // Step 3: Define the required time interval in seconds (30 days = 2,592,000 seconds)
        const SECONDS_IN_A_MONTH: u64 = 30 * 24 * 60 * 60;

        // Step 4: Verify that the required interval has passed since the last mint if current_month > 0
        if emissions_account.current_month > 0 {
            let last_mint_timestamp = emissions_account.last_mint_timestamp.0 / 1_000_000_000; // Convert from nanoseconds to seconds
            log!(
                "Current timestamp: {}, Last mint timestamp: {}, Time passed: {}",
                current_timestamp,
                last_mint_timestamp,
                current_timestamp - last_mint_timestamp
            );

            require!(
                current_timestamp - last_mint_timestamp >= SECONDS_IN_A_MONTH,
                "The required interval has not yet passed"
            );
        }

        // Step 5: Apply decay factor to current_emissions if current_month > 0
        if emissions_account.current_month > 0 {
            emissions_account.current_emissions = U64(
                (emissions_account.current_emissions.0 as f64 * emissions_account.decay_factor)
                    as u64,
            );
        }

        // Step 6: Calculate mint_amount by multiplying current_emissions by 100,000
        let mint_amount = U128(
            u128::from(emissions_account.current_emissions.0) // Explicitly convert u64 to u128
                .checked_mul(100_000)
                .expect("Mint amount multiplication overflow"),
        );

        // Step 7: Execute the mint operation
        self.token.internal_deposit(&self.owner_id, mint_amount.0);

        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &self.owner_id.clone(),
            amount: mint_amount,
            memo: Some("Tokens minted after emissions decay and interval reset"),
        }
        .emit();

        // Step 8: Reset global_tapping_pool.amount
        global_tapping_pool.amount = U128(1_000_000_000_00000);
        self.global_tapping_pool.insert(&2, &global_tapping_pool);

        // Step 9: Apply decay factor to loot_raffle_pool_account.amount if current_month > 0
        if emissions_account.current_month > 0 {
            loot_raffle_pool_account.amount = U128(
                (loot_raffle_pool_account.amount.0 as f64 * emissions_account.decay_factor) as u128,
            );
        }

        // Step 10: Update loot_raffle_pool_account.total_amount
        loot_raffle_pool_account.total_amount = U128(
            loot_raffle_pool_account
                .total_amount
                .0
                .checked_add(loot_raffle_pool_account.amount.0)
                .expect("Total amount addition overflow"),
        );
        self.loot_raffle_pool.insert(&1, &loot_raffle_pool_account);

        // Step 11: Update emissions_account
        emissions_account.last_mint_timestamp = U64(env::block_timestamp()); // Keep this in nanoseconds
        emissions_account.current_month = emissions_account
            .current_month
            .checked_add(1)
            .expect("Current month addition overflow");
        self.emissions_account
            .insert(&self.owner_id, &emissions_account);

        log!("Mint operation completed successfully!");
    }

    pub fn burn(&mut self, amount: U128) {
        // Step 1: Get the caller's account ID
        let caller_id = env::predecessor_account_id();

        // Step 2: Ensure the burn amount is greater than zero
        let burn_amount = amount.0; // Convert U128 to u128
        require!(burn_amount > 0, "Burn amount must be greater than zero");

        // Step 3: Ensure the caller has enough balance to burn
        let caller_balance = self.token.ft_balance_of(caller_id.clone()).0; // Retrieve current balance
        require!(
            caller_balance >= burn_amount,
            format!(
                "Insufficient balance. Available: {}, Required: {}",
                caller_balance, burn_amount
            )
        );

        // Step 4: Withdraw the specified amount from the caller's account
        self.token.internal_withdraw(&caller_id, burn_amount);

        // Step 5: Emit a burn event
        near_contract_standards::fungible_token::events::FtBurn {
            owner_id: &caller_id,
            amount: amount,
            memo: Some("Burning tokens from user's account"),
        }
        .emit();

        // Step 6: Log the burn action for transparency
        log!("{} tokens burned by {}", burn_amount, caller_id);
    }

    #[payable]
    pub fn claim_rewards(
        &mut self,
        amount: U128, // JSON-compatible
        pool_id: u32,
        user_account: AccountId,
    ) {
        let caller_id: AccountId = env::predecessor_account_id();
        log!("Caller ID: {}", caller_id);
        log!("Owner ID: {}", self.owner_id);

        require!(
            caller_id == self.owner_id,
            "Caller is not the contract owner"
        );

        // Step 1: Validate the amount to claim
        let amount_to_claim = amount.0; // Extract raw u128 from U128
        require!(amount_to_claim > 0, "Invalid amount to claim");

        // Step 2: Ensure the user account is registered
        if self
            .token
            .storage_balance_of(user_account.clone())
            .is_none()
        {
            let deposit_amount = self.token.storage_balance_bounds().min;
            require!(
            env::attached_deposit() >= deposit_amount,
            "Attached deposit is less than the minimum storage balance required for account registration"
        );
            self.token.storage_deposit(Some(user_account.clone()), None);
            log!("Storage deposit successful for account: {}", user_account);
        }

        // Step 3: Check and deduct the amount from the respective pool
        match pool_id {
            1 => {
                // Loot Raffle Pool
                let mut loot_pool = self
                    .loot_raffle_pool
                    .get(&1)
                    .expect("Loot Raffle Pool not found");
                require!(
                    amount_to_claim <= loot_pool.amount.0,
                    format!(
                        "Insufficient funds in Loot Raffle Pool. Available: {}, Requested: {}",
                        loot_pool.amount.0, amount_to_claim
                    )
                );
                loot_pool.amount = U128(
                    loot_pool
                        .amount
                        .0
                        .checked_sub(amount_to_claim)
                        .expect("Underflow in Loot Raffle Pool"),
                );
                self.loot_raffle_pool.insert(&1, &loot_pool);
            }
            2 => {
                // Global Tapping Pool
                let mut tapping_pool = self
                    .global_tapping_pool
                    .get(&2)
                    .expect("Global Tapping Pool not found");
                require!(
                    amount_to_claim <= tapping_pool.amount.0,
                    format!(
                        "Insufficient funds in Global Tapping Pool. Available: {}, Requested: {}",
                        tapping_pool.amount.0, amount_to_claim
                    )
                );
                tapping_pool.amount = U128(
                    tapping_pool
                        .amount
                        .0
                        .checked_sub(amount_to_claim)
                        .expect("Underflow in Global Tapping Pool"),
                );
                self.global_tapping_pool.insert(&2, &tapping_pool);
            }
            _ => {
                // Invalid Pool ID
                panic!("Invalid Pool ID");
            }
        }

        // Step 4: Transfer the claimed amount to the user account
        let transfer_amount = amount_to_claim
            .checked_mul(1) // Replace this multiplier with any scaling factor if required
            .expect("Overflow during transfer calculation");

        self.token.ft_transfer(
            user_account,
            U128(transfer_amount),
            Some(format!("Reward claim from pool_id: {}", pool_id)),
        );

        log!(
            "{} tokens claimed from Pool ID: {} by {}",
            transfer_amount,
            pool_id,
            caller_id
        );
    }
}

/// Enforce the requirement of attaching exactly 1 yoctoⓃ for authentication
fn assert_one_yocto() {
    require!(
        env::attached_deposit() == NearToken::from_yoctonear(1),
        "Requires attached deposit of exactly 1 yoctoⓃ for authentication."
    );
}

#[near]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

#[near]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, burned_amount) =
            self.token
                .internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            log!("Account @{} burned {}", sender_id, burned_amount);
        }
        used_amount.into()
    }
}

#[near]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.token.storage_deposit(account_id, registration_only)
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<NearToken>) -> StorageBalance {
        self.token.storage_withdraw(amount)
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        #[allow(unused_variables)]
        if let Some((account_id, balance)) = self.token.internal_storage_unregister(force) {
            log!("Closed @{} with {}", account_id, balance);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.token.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.token.storage_balance_of(account_id)
    }
}

#[near]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use near_sdk::test_utils::{accounts, VMContextBuilder};
//     use near_sdk::testing_env;
//     use near_sdk_contract_tools::owner;

//     fn setup_context(is_view: bool) {
//         let mut builder = VMContextBuilder::new();
//         builder.current_account_id(accounts(0));
//         builder.is_view(is_view);
//         testing_env!(builder.build());
//     }

//     #[test]
//     fn test_new_default_meta() {
//         setup_context(false);

//         let owner_id = accounts(0);
//         let total_supply = U128(1_000_000_000_000); // 1 billion tokens with decimals
//         let contract = Contract::new_default_meta(owner_id.clone(), total_supply);

//         // Verify token metadata
//         let metadata = contract.metadata.get().unwrap();
//         assert_eq!(metadata.name, "PUSH THE BUTTON PTB");
//         assert_eq!(metadata.symbol, "PUSH");
//         assert_eq!(metadata.decimals, 5);

//         // Verify emissions account
//         let emissions_account = contract.emissions_account.get(&owner_id).unwrap();
//         assert_eq!(emissions_account.initial_emissions, 3_000_000_000);
//         assert_eq!(emissions_account.current_emissions, 3_000_000_000);

//         // Verify loot raffle pool
//         let raffle_pool = contract.loot_raffle_pool.get(&1).unwrap();

//         assert_eq!(raffle_pool.pool_id, 1);
//         assert_eq!(raffle_pool.amount, 50_000_000_00000);

//         // Verify global tapping pool
//         let tapping_pool = contract.global_tapping_pool.get(&2).unwrap();
//         assert_eq!(tapping_pool.pool_id, 2);
//         assert_eq!(tapping_pool.amount, 1_000_000_000_00000);

//         // Verify token balance for owner
//         assert_eq!(contract.token.ft_balance_of(owner_id), total_supply);

//         println!("Test passed: `new_default_meta` initialized correctly");
//     }

//     #[test]
//     fn test_mint_function() {
//         setup_context(false);

//         let owner_id = accounts(1);
//         let total_supply = U128(1_000_000_000_000); // Initial supply
//         let mut contract = Contract::new_default_meta(owner_id.clone(), total_supply);

//         // Simulate a passage of time for minting
//         let mut builder = VMContextBuilder::new();
//         builder.current_account_id(accounts(0));
//         builder.block_timestamp(60 * 1_000_000_000); // 1 minute has passed in nanoseconds
//         testing_env!(builder.build());

//         // Perform minting
//         contract.mint(owner_id.clone());

//         // Check updated emissions account
//         let emissions_account = contract.emissions_account.get(&owner_id).unwrap();
//         assert_eq!(emissions_account.current_month, 1); // Current month incremented
//         assert_eq!(
//             emissions_account.current_emissions,
//             3_000_000_000u64
//         ); // Emissions reduced by decay factor
//         assert_eq!(emissions_account.last_mint_timestamp, 60 * 1_000_000_000); // Timestamp updated

//         // Check global tapping pool
//         let tapping_pool = contract.global_tapping_pool.get(&2).unwrap();
//         assert_eq!(tapping_pool.amount, 1_000_000_000_00000); // Reset to default

//         // Check loot raffle pool
//         let raffle_pool = contract.loot_raffle_pool.get(&1).unwrap();
//         assert_eq!(
//             raffle_pool.amount,
//             50_000_000_00000u128
//         ); // Decayed amount

//         // Check owner's new balance
//         let expected_mint_amount = U128(3_000_000_000 as u128 * 100_000); // Adjusted for decimals
//         let new_balance = contract.token.ft_balance_of(owner_id.clone());
//         assert_eq!(
//             new_balance.0,
//             total_supply.0 + expected_mint_amount.0
//         ); // Balance includes minted amount

//         println!("Test passed: `mint` function executed correctly!");
//     }

//     #[test]
//     fn test_claim_rewards() {
//         setup_context(false);

//         let owner_id = accounts(0); // Contract owner
//         let user_id: AccountId = "user1234test.testnet".parse().unwrap(); // Hardcoded user ID
//         let total_supply = U128(1_000_000_000_000); // Initial total supply
//         let mut contract = Contract::new_default_meta(owner_id.clone(), total_supply);

//         // Setup initial pools
//         let initial_loot_pool_amount = 50_000_000_00000;
//         let initial_tapping_pool_amount = 1_000_000_000_00000;

//         // Verify initial loot raffle pool amount
//         let loot_pool = contract.loot_raffle_pool.get(&1).unwrap();
//         assert_eq!(loot_pool.amount, initial_loot_pool_amount);

//         // Verify initial global tapping pool amount
//         let tapping_pool = contract.global_tapping_pool.get(&2).unwrap();
//         assert_eq!(tapping_pool.amount, initial_tapping_pool_amount);

//         // Register user account for rewards (simulate storage deposit)
//         contract.token.internal_register_account(&user_id);

//         // Ensure the owner has sufficient balance
//         let owner_balance = contract.token.ft_balance_of(owner_id.clone()).0;
//         println!("line 502 balance {} {}", owner_balance, owner_id);
//         assert!(
//             owner_balance >= 1,
//             "Owner does not have sufficient balance to claim rewards"
//         );

//         // Ensure the pool has sufficient balance for the claim
//         let claim_amount = 1; // Claim 1 (scaled in the method)
//         let scaled_claim_amount = claim_amount as u128 * 100_000;
//         assert!(
//             loot_pool.amount >= scaled_claim_amount,
//             "Loot pool does not have sufficient balance to satisfy the claim"
//         );

//         // Simulate the required attached deposit of 1 yoctoNEAR
//         let mut builder = VMContextBuilder::new();
//         builder
//             .attached_deposit(NearToken::from_yoctonear(1)) // Attach 1 yoctoNEAR
//             .predecessor_account_id(owner_id.clone()) // Owner is sending rewards
//             .current_account_id(owner_id.clone()); // Contract owner
//         testing_env!(builder.build());

//         // Test claiming rewards from loot raffle pool
//         contract.claim_rewards(claim_amount * 100000, 1, user_id.clone());

//         // Verify updated loot raffle pool
//         let updated_loot_pool = contract.loot_raffle_pool.get(&1).unwrap();
//         assert_eq!(
//             updated_loot_pool.amount,
//             initial_loot_pool_amount - scaled_claim_amount
//         );

//         // Verify user balance after claim
//         let updated_user_balance = contract.token.ft_balance_of(user_id.clone()).0;
//         assert_eq!(updated_user_balance, scaled_claim_amount);

//         println!("Test passed: `claim_rewards` executed correctly!");
//     }

// }

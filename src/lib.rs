use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::{
    FungibleToken, FungibleTokenCore, FungibleTokenResolver,
};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::borsh::BorshSerialize;
use near_sdk::borsh::BorshDeserialize;
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{
    env, log, near, require, AccountId, BorshStorageKey, NearToken, PanicOnDefault, PromiseOrValue,
};
use near_sdk::collections::LookupMap;
use near_sdk_contract_tools::owner;

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct EmissionsAccount {
    pub initial_emissions: u64,
    pub decay_factor: f64,
    pub current_month: u32,
    pub current_emissions: u64,
    pub last_mint_timestamp: u64,
}

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct RafflePool {
    pub pool_id: u32,
    pub amount: u128,
    pub total_amount: u128,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TappingPool {
    pub pool_id: u32,
    pub amount: u128,
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
    proposed_owner: Option<AccountId>, // New field for proposed owner
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "https://red-defensive-termite-556.mypinata.cloud/ipfs/QmUCUAABBsqkhSw3HoeMtecwVAeKBmxUgj2GLwmxuNojbV";

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    FungibleToken,
    Metadata,
}

#[near]
impl Contract {
    /// Initializes the contract with the given total supply 
    #[init]
    pub fn new_default_meta( total_supply: U128) -> Self {
        require!(!env::state_exists(), "Already initialized");
        let caller_id: AccountId = env::predecessor_account_id();
        log!("caller Id  {}", caller_id);

        
        let mut this = Self {

            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "PUSH THE BUTTON PTB".to_string(),
                symbol: "PUSH".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 5,
            })),
            emissions_account: LookupMap::new(b"e"),
            loot_raffle_pool: LookupMap::new(b"l"),
            global_tapping_pool: LookupMap::new(b"g"),
            owner_id: caller_id.clone(),
            proposed_owner: None,

        };
        log!("owner Id  {} ", this.owner_id);
        // Initialize Emissions Account
        this.emissions_account.insert(&this.owner_id, &EmissionsAccount {
            initial_emissions: 3_000_000_000,
            decay_factor: 0.8705505633,
            current_month: 0,
            current_emissions: 3_000_000_000,
            last_mint_timestamp: env::block_timestamp(),
        });

        // Initialize Raffle and Tapping Pools
        this.loot_raffle_pool.insert(&1, &RafflePool {
            pool_id: 1,
            amount: 50_000_000_00000,
            total_amount: 0,
        });
        this.global_tapping_pool.insert(&2, &TappingPool {
            pool_id: 2,
            amount: 1_000_000_000_00000,
        });

        this.token.internal_register_account(&this.owner_id);
        this.token.internal_deposit(&this.owner_id, total_supply.into());

        this
    }

        /// Initiate Ownership Transfer
        #[payable]
        pub fn initiate_ownership_transfer(&mut self, new_owner: AccountId) {
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

        /// Accept Ownership Transfer
        pub fn accept_ownership(&mut self) {
            let proposed_owner = self.proposed_owner.clone().expect("No ownership transfer initiated.");
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
                        initial_emissions: 3_000_000_000,
                        decay_factor: 0.8705505633,
                        current_month: 0,
                        current_emissions: 3_000_000_000,
                        last_mint_timestamp: env::block_timestamp(),
                    },
                );
                log!("Initialized emissions account for new owner: {}", self.owner_id);
            }
        
            // Ensure the new owner has a loot raffle pool account
            if self.loot_raffle_pool.get(&1).is_none() {
                self.loot_raffle_pool.insert(
                    &1,
                    &RafflePool {
                        pool_id: 1,
                        amount: 50_000_000_00000,
                        total_amount: 0,
                    },
                );
                log!("Initialized loot raffle pool for new owner: {}", self.owner_id);
            }
        
            // Ensure the new owner has a global tapping pool account
            if self.global_tapping_pool.get(&2).is_none() {
                self.global_tapping_pool.insert(
                    &2,
                    &TappingPool {
                        pool_id: 2,
                        amount: 1_000_000_000_00000,
                    },
                );
                log!("Initialized global tapping pool for new owner: {}", self.owner_id);
            }
        
            log!("Ownership successfully transferred to: {}", self.owner_id);
        }
    
        /// Check Current and Proposed Owners
        pub fn get_owners(&self) -> (AccountId, Option<AccountId>) {
            log!("Owner ID: {}", self.owner_id.clone());
            log!("Proposed Owner ID: {:?}", self.proposed_owner);
            (self.owner_id.clone(), self.proposed_owner.clone()) // No semicolon here
        }
    

    #[payable]
    // Mint Tokens monthly based on emissions and set pools amount accordingly 
    pub fn mint(&mut self) {
        
        require!(
            env::attached_deposit() == NearToken::from_yoctonear(1),
            "1 yoctoNEAR must be attached for this call"
        );
    
        let caller_id: AccountId = env::predecessor_account_id();
        log!("caller Id  {}", caller_id);
        log!("owner Id  {}", self.owner_id);

        require!(caller_id == self.owner_id, "PTB caller is not the owner");

        // Step 1: Retrieve emissions_account, loot_raffle_pool_account, and global_tapping_pool
        let mut emissions_account = self.emissions_account.get(&self.owner_id.clone())
            .expect("Emissions account not found");
        let mut loot_raffle_pool_account = self.loot_raffle_pool.get(&1)
            .expect("Loot raffle pool account not found");
        let mut global_tapping_pool = self.global_tapping_pool.get(&2)
            .expect("Global tapping pool not found");

        // Step 2: Get the current Unix timestamp
        let current_timestamp = env::block_timestamp();

        // Step 3: Define SECONDS_IN_A_MONTH as 30 days in seconds (2,592,000)
        const SECONDS_IN_A_MONTH: u64 = 30 * 24 * 60 * 60;
        // const SECONDS_IN_A_MONTH: u64 = 60 * 1_000_000_000; // 1 minute in nanoseconds for testing


        // Step 4: Verify that one month has passed since the last mint if current_month > 0
        if emissions_account.current_month > 0 {
            require!(
                current_timestamp >= emissions_account.last_mint_timestamp + SECONDS_IN_A_MONTH,
                "Month has not yet elapsed"
            );
        }

        // Step 5: Apply decay factor to current_emissions if current_month > 0
        if emissions_account.current_month > 0 {
            emissions_account.current_emissions = (emissions_account.current_emissions as f64 * emissions_account.decay_factor) as u64;
        }

        // Step 6: Log the owner of the mint account
        // log!("Mint account owner: {}", owner_id);

        // Step 8: Calculate mint_amount by multiplying current_emissions by 100,000 to adjust for decimals
        let mint_amount = U128(emissions_account.current_emissions as u128 * 100_000);

        // Step 9: Execute the mint operation
        self.token.internal_deposit(&self.owner_id, mint_amount.0);

        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &self.owner_id.clone(),
            amount: mint_amount,
            memo: Some("tokens minted after emissions decay and monthly reset"),
        }
        .emit();

        // Step 10: Reset global_tapping_pool.amount to 1,000,000,000,00000
        global_tapping_pool.amount = 1_000_000_000_00000;
        self.global_tapping_pool.insert(&2, &global_tapping_pool);

        // Step 11: Apply decay factor to loot_raffle_pool_account.amount if current_month > 0
        if emissions_account.current_month > 0 {
            loot_raffle_pool_account.amount = (loot_raffle_pool_account.amount as f64 * emissions_account.decay_factor) as u128;
        }

        // Step 12: Add the decayed loot_raffle_pool_account.amount to total_amount
        loot_raffle_pool_account.total_amount += loot_raffle_pool_account.amount;
        self.loot_raffle_pool.insert(&1, &loot_raffle_pool_account);

        // Step 13: Update last_mint_timestamp in emissions_account to the current timestamp
        emissions_account.last_mint_timestamp = current_timestamp;

        // Step 14: Increment current_month in emissions_account by 1
        emissions_account.current_month += 1;
        self.emissions_account.insert(&self.owner_id, &emissions_account);
    } 

    pub fn burn(&mut self, amount: U128) {
        // Step 1: Get the caller's account ID
        let caller_id = env::predecessor_account_id();
    
        // Step 2: Ensure the caller has enough balance to burn
        let burn_amount = amount.0; // Convert U128 to u128
        require!(
            burn_amount > 0,
            "Burn amount must be greater than zero"
        );
    
        // Step 3: Withdraw the specified amount from the caller's account
        self.token.internal_withdraw(&caller_id, burn_amount);
    
        // Step 4: Emit a burn event
        near_contract_standards::fungible_token::events::FtBurn {
            owner_id: &caller_id,
            amount: amount,
            memo: Some("Burning tokens from user's account"),
        }
        .emit();
    
        // Log the burn action for transparency
        log!("{} tokens burned by {}", burn_amount, caller_id);
    }
    

    
    #[payable]
    pub fn claim_rewards(
        &mut self,
        amount: u64,
        pool_id: u32,
        user_account: AccountId,
    ) {

        let caller_id: AccountId = env::predecessor_account_id();
        log!("caller Id  {}", caller_id);
        log!("owner Id  {}", self.owner_id);

        require!(caller_id == self.owner_id, "PTB caller is not the owner");

        // Step 1: Validate the amount to claim
        require!(amount > 0, "Invalid Amount".to_string());

        // Step 2: Ensure the user account is registered
        if self.token.storage_balance_of(user_account.clone()).is_none() {
            // log!("User account not registered, performing storage deposit.");
            let deposit_amount = self.token.storage_balance_bounds().min;
            // log!("deposit amount {} ", deposit_amount);
            // log!("env deposit amount {} ", env::attached_deposit());

            require!(
                env::attached_deposit() >= deposit_amount,
                "Attached deposit is less than the minimum storage balance"
            );
    
            self.token.storage_deposit(Some(user_account.clone()), None);
            // log!("Storage deposit successful for account: {}", user_account);
        }
    
        // Step 3: Scale the input amount
        let amount_to_claim = amount as u128 ;
    
        match pool_id {
            1 => {
                // Step 4a: Check and deduct from loot raffle pool
                let mut loot_pool = self
                    .loot_raffle_pool
                    .get(&1)
                    .expect("Loot Raffle Pool not found");
                require!(
                    amount_to_claim <= loot_pool.amount,
                    "Insufficient funds in Loot Raffle Pool".to_string()
                );
                loot_pool.amount = loot_pool
                    .amount
                    .checked_sub(amount_to_claim)
                    .expect("Underflow in Loot Raffle Pool");
                self.loot_raffle_pool.insert(&1, &loot_pool);
            }
            2 => {
                // Step 4b: Check and deduct from global tapping pool
                let mut tapping_pool = self
                    .global_tapping_pool
                    .get(&2)
                    .expect("Global Tapping Pool not found");
                require!(
                    amount_to_claim <= tapping_pool.amount,
                    "Insufficient funds in Global Tapping Pool".to_string()
                );
                tapping_pool.amount = tapping_pool
                    .amount
                    .checked_sub(amount_to_claim)
                    .expect("Underflow in Global Tapping Pool");
                self.global_tapping_pool.insert(&2, &tapping_pool);
            }
            _ => {
                // Step 4c: Handle invalid pool ID
                panic!("Invalid Pool ID");
            }
        }
    
        // Step 5: Log associated accounts for debugging
        // log!("User Account: {}", user_account);
    
        // Step 6: Execute the token transfer
        self.token.ft_transfer(
            user_account,
            U128(amount_to_claim),
            Some(format!("Reward claim from pool_id: {}", pool_id)),
        );
    }
            
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
            self.token.internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
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

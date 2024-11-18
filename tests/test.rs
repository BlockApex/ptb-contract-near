// #[cfg(test)]
// mod tests {
//     use super::*;
//     use near_sdk::test_utils::{accounts, VMContextBuilder};
//     use near_sdk::testing_env;

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
//             (3_000_000_000u64 as f64 * 0.8705505633) as u64
//         ); // Emissions reduced by decay factor
//         assert_eq!(emissions_account.last_mint_timestamp, 60 * 1_000_000_000); // Timestamp updated
    
//         // Check global tapping pool
//         let tapping_pool = contract.global_tapping_pool.get(&2).unwrap();
//         assert_eq!(tapping_pool.amount, 1_000_000_000_00000); // Reset to default
    
//         // Check loot raffle pool
//         let raffle_pool = contract.loot_raffle_pool.get(&1).unwrap();
//         assert_eq!(
//             raffle_pool.amount,
//             (50_000_000_00000u64 as f64 * 0.8705505633) as u128
//         ); // Decayed amount
//         assert_eq!(
//             raffle_pool.total_amount,
//             raffle_pool.amount + (50_000_000_00000u64 as f64 * 0.8705505633) as u128
//         ); // Updated total amount
    
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
//         let mut builder = VMContextBuilder::new();
//         builder
//             .current_account_id(accounts(0))
//             .attached_deposit(NearToken::from_yoctonear(1)); // Attach 1 yoctoNEAR
//         testing_env!(builder.build());
    
//         let owner_id = accounts(1);
//         let associated_account = accounts(2);
//         let user_account = accounts(3); // User account to receive rewards
//         let token_address = accounts(0); // Token address matches the contract's account
//         let total_supply = U128(1_000_000_000_000); // Initial supply
//         let mut contract = Contract::new_default_meta(owner_id.clone(), total_supply);
    
//         // Step 1: Register the user account
//         builder.attached_deposit(NearToken::from_yoctonear(1250000000000000000000)); // Attach sufficient deposit for storage
//         testing_env!(builder.build());
//         contract.token.storage_deposit(Some(user_account.clone()), None);
    
//         // Step 2: Simulate adding some tokens to the pools
//         contract.loot_raffle_pool.insert(
//             &1,
//             &RafflePool {
//                 pool_id: 1,
//                 amount: 50_000_000_00000,
//                 total_amount: 0,
//             },
//         );
    
//         contract.global_tapping_pool.insert(
//             &2,
//             &TappingPool {
//                 pool_id: 2,
//                 amount: 1_000_000_000_00000,
//             },
//         );
    
//         // Step 3: Attempt to claim from loot raffle pool
//         let claim_amount = 10_000; // Example claim amount
//         contract.claim_rewards(
//             owner_id.clone(),
//             claim_amount,
//             1, // Loot raffle pool
//             token_address.clone(),
//             associated_account.clone(),
//             user_account.clone(),
//         );
    
//         // Step 4: Verify updated loot raffle pool
//         let updated_loot_pool = contract.loot_raffle_pool.get(&1).unwrap();
//         let expected_remaining_amount = 50_000_000_00000 - (claim_amount as u128 * 100_000);
//         assert_eq!(updated_loot_pool.amount, expected_remaining_amount);
    
//         // Step 5: Verify user's token balance
//         let user_balance = contract.token.ft_balance_of(user_account.clone());
//         assert_eq!(user_balance, U128(claim_amount as u128 * 100_000));
    
//         println!("Test passed: `claim_rewards` executed correctly!");
//     }
        
// }

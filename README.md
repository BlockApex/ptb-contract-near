# PTB Token Smart Contract

The PTB project implements a NEAR-based fungible token with minting, burning, and reward claim functionality. The token has a decimal precision of 5. Below are the primary features and the commands required to deploy and interact with the contract.

## Features

1. **Token Initialization**:
The new_default_meta function initializes the smart contract with default metadata, sets up initial accounts, and establishes the structure for token emissions and reward pools. This function can only be called once and requires the caller to be the contract owner. It initializes a fungible token with specified metadata, including the token name, symbol, decimals, and icon. The function also creates an emissions account for the owner with an initial emissions amount of 3,000,000,000, a decay factor of 0.8705505633, and sets the starting timestamp for minting. Additionally, it initializes two pools: the loot raffle pool with an initial amount of 50,000,000,00000 and the global tapping pool with 1,000,000,000,00000. Lastly, it registers the owner’s account and deposits the specified total_supply into it.

2. **Mint Functionality**:
The mint function mints tokens for the contract owner on a monthly basis, applying a decay factor to reduce emissions over time. It ensures only the owner can call this function, verifies that one month has passed since the last mint, and calculates the new token amount (adjusted for 5 decimals) to deposit into the owner’s account. Additionally, it resets the global tapping pool, updates the loot raffle pool with decayed values, and logs the minting process. The function also updates the last mint timestamp and increments the mint cycle to track progress.


3. **Burn Functionality**:
   - Allows the token owner to burn tokens, effectively removing them from circulation.

4. **Reward Claim**:
The claim_rewards function allows the contract owner to distribute rewards to a specific user account from one of the predefined pools (loot raffle pool or global tapping pool). The function validates the requested reward amount, ensures the user's account is registered for storage, and checks whether the specified pool contains sufficient funds to cover the claim. If the user account is not registered, the function performs a storage deposit using the attached deposit. The function deducts the claimed amount from the specified pool and transfers the tokens to the user's account. It also ensures that the claim is only executed by the contract owner and handles errors such as insufficient funds or invalid pool IDs. This function is marked as #[payable] to allow attaching a deposit for user account storage registration.



---

## Setup and Deployment

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) and Cargo installed.
- NEAR CLI installed (`npm install -g near-cli`).

### Commands

#### 1. Build the Contract
Clean and build the contract:
```bash
cargo clean


cargo build --target wasm32-unknown-unknown --release


near contract deploy ptbfinaltest4.testnet use-file target/wasm32-unknown-unknown/release/near_contract_project.wasm with-init-call new_default_meta json-args '{"owner_id": "ptbfinaltest4.testnet", "total_supply": "10000000000"}' prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send^C

near contract call-function as-transaction ptbfinaltest4.testnet mint json-args {} prepaid-gas '100.0 Tgas' attached-deposit '1 yoctoNEAR' sign-as ptbfinaltest4.testnet network-config testnet sign-with-keychain send

near contract call-function as-transaction ptbtest1234.testnet storage_deposit json-args '{"account_id": "user1234test.testnet"}' prepaid-gas '100.0 Tgas' attached-deposit '0.01 NEAR' sign-as ptbtest1234.testnet network-config testnet sign-with-keychain send
   
near contract call-function as-transaction ptbtestptb1.testnet claim_rewards json-args '{"amount": "10", "pool_id": 1, "user_account": "user1234test.testnet"}' prepaid-gas '100.0 Tgas' attached-deposit '1 yoctoNEAR' sign-as ptbtestptb1.testnet network-config testnet sign-with-keychain send

near contract call-function as-transaction ptbfinaltest2.testnet initiate_ownership_transfer json-args '{"new_owner":"ptbfinaltest3.testnet"}' prepaid-gas '100.0 Tgas' attached-deposit '1 yoctoNEAR' sign-as ptbfinaltest2.testnet network-config testnet sign-with-keychain send

near contract call-function as-transaction ptbfinaltest2.testnet get_owners json-args {} prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as ptbfinaltest2.testnet network-config testnet sign-with-keychain send

near contract call-function as-transaction ptbfinaltest2.testnet accept_ownership json-args {} prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as ptbfinaltest3.testnet network-config testnet sign-with-keychain send

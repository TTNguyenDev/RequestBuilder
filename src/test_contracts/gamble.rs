
/// Enable the user to transfer near to the contract to get a chance to throw the dice
/// When 6 is hit, Users will get @Factor times the amount of near transfered
/// 
/// @Author Young
/// @Date 2022.05.05
/// 

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen, Balance,PanicOnDefault,
    env, require, log,  Promise, 
};

// Tax collected per success throw 
const TAX : f32 = 0.95;
// When you throw the dice and get 6, you will get FACTOR * the bet you transfer to the contract
const FACTOR: u128 = 6;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize,PanicOnDefault)]
pub struct Gamble {

    // Minimum price that should be transfered to the contract, revert otherwise
    gamble_min_price : Balance,

    // Maximum price that should be transfered to the contract, revert otherwise
    gamble_max_price : Balance,

}


#[near_bindgen]
impl Gamble {
    
    // The new function should be called to initialize the contract, and set the gamble_max_price and the gamble_min_price
    #[init]
    pub fn new() -> Self {
        
        let account_balance = env::account_balance();
        let gamble_max_price = account_balance / (5 * FACTOR);
        log!("we have {} uints in total, be sure not to exceed the max gamble price limit {} to get {}X \n", account_balance, gamble_max_price, FACTOR);

        Self{
            gamble_max_price : gamble_max_price,
            gamble_min_price : 0,
        }
    }

    // Get the Minimum amount of near to be transfered(Used for dapp, but usually won't as it's 0 all the time)
    pub fn get_minimal_gamble_price(&self) -> u128 {
        self.gamble_min_price
    }

    // Get the Minimum amount of near to be transfered(Used for dapp)
    pub fn get_maximum_gamble_price(&self) -> u128 {
        self.gamble_max_price
    }    

    // Get contract balance U128
    pub fn get_balance(&self) -> u128 {
        env::account_balance()
    }

    // Update price everytime the account balance changes
    // Only contract call
    fn update_price(&mut self){
        let account_balance = env::account_balance();
        self.gamble_max_price = account_balance / (5 * FACTOR);
        log!("we have {} uints in total, be sure not to exceed the max gamble price limit {} to get {}X \n", account_balance, self.gamble_max_price, FACTOR);
    }

    // The user could sponsor the contract(maybe only the owner will...)
    #[payable]
    pub fn sponsor(&mut self){
        let sponsor_id = env::signer_account_id();
        let deposit = env::attached_deposit();
        log!("sponsor {} has add {} to the game to increase balance, thank you ~ \n", sponsor_id, deposit);
        self.update_price();
    }

    // The user could transfer near to get a chance to gamble
    // return the dice throwed by the user (Randomly generated)
    #[payable]
    pub fn gamble(&mut self) -> u8{
        let gambler_id = env::signer_account_id();
        let deposit = env::attached_deposit();

        require!(deposit>=self.gamble_min_price,"The gamble price must exceed gamble_min_price");
        require!(deposit<=self.gamble_max_price,"The gamble price must not exceed gamble_max_price");
        
        let num = self.rand_dice();

        if num == FACTOR as u8 {
            let amount = (deposit as f32 ) *(FACTOR as f32) * TAX;
            let amount_u128 = amount  as u128;
            log!("Congratuations to {}, he has won the gamble, the prize is {} \n",gambler_id,deposit);
            Promise::new(gambler_id).transfer(amount_u128);
        }
        self.update_price();
        return num;
    }

    // Generate random number from 1 to 6
    pub fn rand_dice(&self) -> u8 {
        *env::random_seed().get(0).unwrap()%6+1
    }

}


#[cfg(not(target_arch="wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{testing_env,VMContext};
    use near_sdk::Gas;
    use near_sdk::AccountId;

    fn get_context(input: Vec<u8>) -> VMContext {
        VMContext {
            current_account_id: AccountId::new_unchecked("alice.testnet".to_string()),
            signer_account_id: AccountId::new_unchecked("robert.testnet".to_string()),
            signer_account_pk: vec![0u8; 33].try_into().unwrap(),
            predecessor_account_id: AccountId::new_unchecked("jane.testnet".to_string()),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 222,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: Gas(10u64.pow(18)),
            random_seed: [5u8; 32],
            view_config: None,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn rand_test() {
        let context = get_context(vec![]);
        testing_env!(context);
        let contract = Gamble{
            gamble_min_price : 0,
            gamble_max_price : 0,
        };
        let val = contract.rand_dice();
        println!("{}",val);
        assert_eq!(val>=1,true,"The random value should not be smaller than 1");
        assert_eq!(val<=6,true,"The random value should not be bigger than 6");
        
    }

    #[test]
    fn gamble_test() {
        let context = get_context(vec![]);
        testing_env!(context);
        let mut contract = Gamble{
            gamble_min_price : 0,
            gamble_max_price : 0,
        };
        println!("minimal : {}",contract.get_minimal_gamble_price());
        println!("maximum : {}",contract.get_maximum_gamble_price());
        println!("balance : {}",contract.get_balance());
        println!("gamble: {}",contract.gamble());
    }
}

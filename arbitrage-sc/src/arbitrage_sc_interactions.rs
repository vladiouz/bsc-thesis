use crate::arbitrage_sc_requirements;
use crate::arbitrage_sc_storage;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ArbitrageScInteractions:
    arbitrage_sc_requirements::ArbitrageScRequirements + arbitrage_sc_storage::ArbitrageScStorage
{
    #[payable("*")]
    #[endpoint(stake)]
    fn stake(&self) {
        self.require_is_active();
        self.require_staking_active();

        let payment = self.call_value().single_esdt();

        require!(
            payment.token_identifier == self.staked_token_id().get(),
            "Wrong token for stake"
        );
        require!(payment.token_nonce == 0, "Token must be fungible");

        let caller = self.blockchain().get_caller();

        let mut curr_amount = self.staked_amount().get(&caller).unwrap_or_default();
        curr_amount += &payment.amount;
        self.staked_amount().insert(caller, curr_amount);
    }

    #[endpoint(unstake)]
    fn unstake(&self) {
        let user = self.blockchain().get_caller();
        let user_stake = self.staked_amount().get(&user).unwrap_or_default();
        let user_winnings = self.user_winnings(user.clone()).get();

        require!(user_stake > 0, "No stake to unstake");

        let total_payout = user_stake + user_winnings;
        let staked_token_id = self.staked_token_id().get();

        self.send()
            .direct_esdt(&user, &staked_token_id, 0, &total_payout);

        self.staked_amount().remove(&user);
        self.user_winnings(user).set(BigUint::zero());
    }

    #[endpoint(restakeWinnings)]
    fn restake_winnings(&self) {
        self.require_is_active();
        self.require_staking_active();

        let user = self.blockchain().get_caller();
        let user_winnings = self.user_winnings(user.clone()).get();

        require!(user_winnings > 0, "No winnings to restake");

        let mut user_stake = self.staked_amount().get(&user).unwrap_or_default();

        user_stake += &user_winnings;
        self.staked_amount().insert(user.clone(), user_stake);
        self.user_winnings(user).set(BigUint::zero());
    }

    #[endpoint(claimWinnings)]
    fn claim_winnings(&self) {
        let user = self.blockchain().get_caller();
        let user_winnings = self.user_winnings(user.clone()).get();

        require!(user_winnings > 0, "No winnings to claim");

        let staked_token_id = self.staked_token_id().get();
        self.send()
            .direct_esdt(&user, &staked_token_id, 0, &user_winnings);
        self.user_winnings(user).set(BigUint::zero());
    }
}

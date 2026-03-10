use crate::arbitrage_sc_requirements;
use crate::arbitrage_sc_storage;
use crate::pair_proxy;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

type SwapOperation<M> = MultiValue2<ManagedAddress<M>, TokenIdentifier<M>>;

#[multiversx_sc::module]
pub trait ArbitrageScOwnerInteractions:
    arbitrage_sc_storage::ArbitrageScStorage + arbitrage_sc_requirements::ArbitrageScRequirements
{
    #[only_owner]
    #[endpoint(pause)]
    fn pause(&self) {
        self.is_paused().set(true);
    }

    #[only_owner]
    #[endpoint(unpause)]
    fn unpause(&self) {
        self.is_paused().set(false);
    }

    #[only_owner]
    #[endpoint(setStakedToken)]
    fn set_staked_token(&self, token_id: TokenIdentifier) {
        self.staked_token_id().set_if_empty(token_id);
    }

    #[only_owner]
    #[endpoint(setOwnerWinningsPercentage)]
    fn set_owner_winnings_percentage(&self, percentage: u8) {
        require!(
            percentage <= 100u8,
            "Percentage cannot be greater than 100%"
        );
        self.owner_winnings_percentage()
            .set_if_empty(BigUint::from(percentage));
    }

    #[only_owner]
    #[endpoint(withdrawDevWinnings)]
    fn withdraw_dev_winnings(&self) {
        let winnings = self.dev_winnings().get();
        require!(winnings > 0, "No winnings to withdraw");

        let staked_token_id = self.staked_token_id().get();
        self.dev_winnings().set(BigUint::zero());

        self.send().direct_esdt(
            &self.blockchain().get_owner_address(),
            &staked_token_id,
            0,
            &winnings,
        );
    }

    #[only_owner]
    #[endpoint(executeTrades)]
    fn execute_trades(&self, swaps: MultiValueEncoded<SwapOperation<Self::Api>>) {
        self.require_is_active();

        let mut last_amount = BigUint::from(1_000_000u32);
        let min_amount_out = BigUint::from(1u32);
        let mut token_in = self.staked_token_id().get();

        let staked_token_reserve = self.blockchain().get_sc_balance(
            TokenId::from(EgldOrEsdtTokenIdentifier::esdt(token_in.clone())),
            0,
        );

        for swap in swaps.into_iter() {
            let (pair_address, token_out) = swap.into_tuple();

            require!(
                self.blockchain().is_smart_contract(&pair_address),
                "Pair address is not a smart contract"
            );

            self.tx()
                .to(pair_address)
                .gas(20_000_000u64)
                .typed(pair_proxy::PairProxy)
                .swap_tokens_fixed_input(token_out.clone(), min_amount_out.clone())
                .with_esdt_transfer((token_in, 0, last_amount.clone()))
                .sync_call();

            last_amount = self.blockchain().get_sc_balance(
                TokenId::from(EgldOrEsdtTokenIdentifier::esdt(token_out.clone())),
                0,
            );
            token_in = token_out;
        }

        require!(
            token_in == self.staked_token_id().get(),
            "The trade path must be a cycle"
        );
        require!(
            last_amount > staked_token_reserve,
            "Trade is not profitable"
        );

        let profit = &last_amount - &staked_token_reserve;
        let mut total_staked_amount = BigUint::zero();

        for (_, amount) in self.staked_amount().iter() {
            total_staked_amount += amount;
        }

        let owner_profit = &profit * self.owner_winnings_percentage().get() / 100u32;
        self.dev_winnings()
            .set(self.dev_winnings().get() + &owner_profit);

        let users_profit = &profit - &owner_profit;

        for (staker, amount) in self.staked_amount().iter() {
            let staker_profit = &users_profit * amount / &total_staked_amount;
            self.user_winnings(staker.clone())
                .set(self.user_winnings(staker).get() + staker_profit);
        }
    }
}

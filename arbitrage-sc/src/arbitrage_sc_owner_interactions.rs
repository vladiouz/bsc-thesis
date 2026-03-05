use crate::arbitrage_sc_storage;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ArbitrageScOwnerInteractions: arbitrage_sc_storage::ArbitrageScStorage {
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
    #[endpoint(withdrawDevWinnings)]
    fn withdraw_dev_winnings(&self) {
        // to impl
    }

    #[only_owner]
    #[endpoint(executeTrades)]
    fn execute_trades(
        &self,
        token_out: TokenIdentifier,
        min_amount_out: BigUint,
        sc: ManagedAddress,
    ) {
        self.tx()
            .to(sc)
            .raw_call("swapTokensFixedInput")
            .argument(&token_out)
            .argument(&min_amount_out)
            .with_esdt_transfer((self.staked_token_id().get(), 0, BigUint::from(1_000_000u32)))
            .async_call_and_exit();
    }
}

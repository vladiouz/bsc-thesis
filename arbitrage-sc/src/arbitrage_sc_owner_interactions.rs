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
    #[endpoint(withdrawDevWinnings)]
    fn withdraw_dev_winnings(&self) {
        // to impl
    }

    #[only_owner]
    #[endpoint(executeTrades)]
    fn execute_trades(&self, swaps: MultiValueEncoded<SwapOperation<Self::Api>>) {
        self.require_is_active();

        let mut last_amount = BigUint::from(1_000_000u32);
        let min_amount_out = BigUint::from(1u32);
        let mut token_in = self.staked_token_id().get();

        for swap in swaps.into_iter() {
            let (pair_address, token_out) = swap.into_tuple();

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
    }
}

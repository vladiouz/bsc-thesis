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

        let payment = self.call_value().single_esdt();

        require!(
            payment.token_identifier == self.staked_token_id().get(),
            "Wrong token for stake"
        );

        let caller = self.blockchain().get_caller();

        let mut curr_amount = self.staked_amount().get(&caller).unwrap_or_default();
        curr_amount += &payment.amount;
        self.staked_amount().insert(caller, curr_amount);
    }
}

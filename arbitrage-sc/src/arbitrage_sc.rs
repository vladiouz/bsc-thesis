#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod arbitrage_sc_interactions;
pub mod arbitrage_sc_owner_interactions;
pub mod arbitrage_sc_requirements;
pub mod arbitrage_sc_storage;
pub mod pair_proxy;

#[multiversx_sc::contract]
pub trait ArbitrageSc:
    arbitrage_sc_storage::ArbitrageScStorage
    + arbitrage_sc_owner_interactions::ArbitrageScOwnerInteractions
    + arbitrage_sc_interactions::ArbitrageScInteractions
    + arbitrage_sc_requirements::ArbitrageScRequirements
{
    #[init]
    fn init(&self) {
        self.is_paused().set(true);
        self.dev_winnings().set(BigUint::zero());
    }

    #[upgrade]
    fn upgrade(&self) {}
}

#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod arbitrage_sc_owner_interactions;
pub mod arbitrage_sc_storage;

#[multiversx_sc::contract]
pub trait ArbitrageSc:
    arbitrage_sc_storage::ArbitrageScStorage
    + arbitrage_sc_owner_interactions::ArbitrageScOwnerInteractions
{
    #[init]
    fn init(&self) {
        self.is_paused().set(true);
        self.dev_winnings().set(BigUint::zero());
    }

    #[upgrade]
    fn upgrade(&self) {}
}

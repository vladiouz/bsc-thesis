#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod arbitrage_sc_storage;

#[multiversx_sc::contract]
pub trait ArbitrageSc: arbitrage_sc_storage::ArbitrageScStorage {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}

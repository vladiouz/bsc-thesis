use crate::arbitrage_sc_storage;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ArbitrageScRequirements: arbitrage_sc_storage::ArbitrageScStorage {
    fn require_is_active(&self) {
        require!(!self.is_paused().get(), "Contract is paused")
    }
}

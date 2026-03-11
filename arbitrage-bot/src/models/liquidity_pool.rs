use crate::DEFAULT_FEE;
use num_bigint::BigUint;
use num_traits::Zero;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct LiquidityPool {
    pub sc_address: String,
    pub base_id: String,
    pub quote_id: String,
    pub base_reserve: BigUint,
    pub quote_reserve: BigUint,
    pub fee: u32,
}

impl LiquidityPool {
    pub fn new(sc_address: String, base_id: String, quote_id: String) -> Self {
        LiquidityPool {
            sc_address,
            base_id,
            quote_id,
            base_reserve: Zero::zero(),
            quote_reserve: Zero::zero(),
            fee: DEFAULT_FEE,
        }
    }
}

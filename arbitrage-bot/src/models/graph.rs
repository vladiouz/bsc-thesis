use crate::models::liquidity_pool::LiquidityPool;
use num_bigint::BigUint;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Edge {
    pub sc_address: String,
    pub out_id: String,
    pub in_reserve: BigUint,
    pub out_reserve: BigUint,
    pub fee: u32,
}

pub type Graph = HashMap<String, Vec<Edge>>;

pub fn build_graph(pools: &Vec<LiquidityPool>) -> Graph {
    let mut graph: Graph = HashMap::new();

    for pool in pools {
        graph.entry(pool.base_id.clone()).or_default().push(Edge {
            sc_address: pool.sc_address.clone(),
            out_id: pool.quote_id.clone(),
            in_reserve: pool.base_reserve.clone(),
            out_reserve: pool.quote_reserve.clone(),
            fee: pool.fee,
        });

        graph.entry(pool.quote_id.clone()).or_default().push(Edge {
            sc_address: pool.sc_address.clone(),
            out_id: pool.base_id.clone(),
            in_reserve: pool.quote_reserve.clone(),
            out_reserve: pool.base_reserve.clone(),
            fee: pool.fee,
        });
    }

    graph
}

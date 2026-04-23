use crate::DEFAULT_FEE;
use num_bigint::BigUint;
use num_traits::Zero;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

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

pub fn filter_pools(pools: Vec<LiquidityPool>, base_token_id: &str) -> Vec<LiquidityPool> {
    let mut token_graph: HashMap<String, HashSet<String>> = HashMap::new();
    let mut pool_map: HashMap<(String, String), usize> = HashMap::new();

    for (idx, pool) in pools.iter().enumerate() {
        token_graph
            .entry(pool.base_id.clone())
            .or_default()
            .insert(pool.quote_id.clone());
        token_graph
            .entry(pool.quote_id.clone())
            .or_default()
            .insert(pool.base_id.clone());

        pool_map.insert((pool.base_id.clone(), pool.quote_id.clone()), idx);
        pool_map.insert((pool.quote_id.clone(), pool.base_id.clone()), idx);
    }

    let Some(base_neighbors) = token_graph.get(base_token_id) else {
        return Vec::new();
    };

    let mut selected_indices: HashSet<usize> = HashSet::new();

    for token_a in base_neighbors {
        if token_a == base_token_id {
            continue;
        }

        let Some(token_a_neighbors) = token_graph.get(token_a) else {
            continue;
        };

        for token_b in token_a_neighbors {
            if token_b == base_token_id || token_b == token_a {
                continue;
            }

            let closes_triangle = token_graph
                .get(token_b)
                .map(|neighbors| neighbors.contains(base_token_id))
                .unwrap_or(false);

            if !closes_triangle {
                continue;
            }

            if let Some(idx) = pool_map.get(&(base_token_id.to_string(), token_a.clone())) {
                selected_indices.insert(*idx);
            }
            if let Some(idx) = pool_map.get(&(token_a.clone(), token_b.clone())) {
                selected_indices.insert(*idx);
            }
            if let Some(idx) = pool_map.get(&(token_b.clone(), base_token_id.to_string())) {
                selected_indices.insert(*idx);
            }
        }
    }

    pools
        .into_iter()
        .enumerate()
        .filter_map(|(idx, pool)| selected_indices.contains(&idx).then_some(pool))
        .collect()
}

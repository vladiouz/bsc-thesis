use crate::config::DEFAULT_FEE;
use num_bigint::BigUint;
use num_traits::Zero;
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Serialize, Debug, Clone)]
pub struct LiquidityPool {
    pub sc_address: String,
    pub base_id: String,
    pub quote_id: String,
    pub base_is_first: bool,
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
            base_is_first: true,
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

fn build_token_graph_undirected(pools: &[LiquidityPool]) -> HashMap<String, HashSet<String>> {
    let mut token_graph: HashMap<String, HashSet<String>> = HashMap::new();

    for pool in pools {
        token_graph
            .entry(pool.base_id.clone())
            .or_default()
            .insert(pool.quote_id.clone());
        token_graph
            .entry(pool.quote_id.clone())
            .or_default()
            .insert(pool.base_id.clone());
    }

    token_graph
}

fn collect_component_tokens(
    token_graph: &HashMap<String, HashSet<String>>,
    base_token_id: &str,
) -> HashSet<String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    if !token_graph.contains_key(base_token_id) {
        return visited;
    }

    queue.push_back(base_token_id.to_string());
    visited.insert(base_token_id.to_string());

    while let Some(token) = queue.pop_front() {
        if let Some(neighbors) = token_graph.get(&token) {
            for neighbor in neighbors {
                if visited.insert(neighbor.clone()) {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    visited
}

pub fn filter_pools_by_base_component(
    pools: Vec<LiquidityPool>,
    base_token_id: &str,
) -> Vec<LiquidityPool> {
    let token_graph = build_token_graph_undirected(&pools);
    let component_tokens = collect_component_tokens(&token_graph, base_token_id);

    if component_tokens.is_empty() {
        return Vec::new();
    }

    pools
        .into_iter()
        .filter(|pool| {
            component_tokens.contains(&pool.base_id) && component_tokens.contains(&pool.quote_id)
        })
        .collect()
}

pub fn filter_pools_by_max_hops(
    pools: Vec<LiquidityPool>,
    base_token_id: &str,
    max_hops: usize,
) -> Vec<LiquidityPool> {
    let token_graph = build_token_graph_undirected(&pools);
    if !token_graph.contains_key(base_token_id) {
        return Vec::new();
    }

    let mut distance: HashMap<String, usize> = HashMap::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(base_token_id.to_string());
    distance.insert(base_token_id.to_string(), 0);

    while let Some(token) = queue.pop_front() {
        let hop = *distance.get(&token).unwrap_or(&usize::MAX);
        if hop >= max_hops {
            continue;
        }

        if let Some(neighbors) = token_graph.get(&token) {
            for neighbor in neighbors {
                if !distance.contains_key(neighbor) {
                    distance.insert(neighbor.clone(), hop + 1);
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    pools
        .into_iter()
        .filter(|pool| {
            distance.contains_key(&pool.base_id) && distance.contains_key(&pool.quote_id)
        })
        .collect()
}

pub fn filter_pools_for_cycle_search(
    pools: Vec<LiquidityPool>,
    base_token_id: &str,
) -> Vec<LiquidityPool> {
    let token_graph = build_token_graph_undirected(&pools);
    let component_tokens = collect_component_tokens(&token_graph, base_token_id);
    if component_tokens.is_empty() {
        return Vec::new();
    }

    let mut subgraph: HashMap<String, HashSet<String>> = HashMap::new();
    for token in &component_tokens {
        let mut neighbors_in_component = HashSet::new();
        if let Some(neighbors) = token_graph.get(token) {
            for neighbor in neighbors {
                if component_tokens.contains(neighbor) {
                    neighbors_in_component.insert(neighbor.clone());
                }
            }
        }
        subgraph.insert(token.clone(), neighbors_in_component);
    }

    let mut queue: VecDeque<String> = subgraph
        .iter()
        .filter_map(|(token, neighbors)| (neighbors.len() < 2).then_some(token.clone()))
        .collect();

    while let Some(token) = queue.pop_front() {
        let Some(neighbors) = subgraph.remove(&token) else {
            continue;
        };

        for neighbor in neighbors {
            if let Some(neighbor_set) = subgraph.get_mut(&neighbor) {
                neighbor_set.remove(&token);
                if neighbor_set.len() < 2 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    if !subgraph.contains_key(base_token_id) {
        return Vec::new();
    }

    pools
        .into_iter()
        .filter(|pool| {
            subgraph.contains_key(&pool.base_id) && subgraph.contains_key(&pool.quote_id)
        })
        .collect()
}

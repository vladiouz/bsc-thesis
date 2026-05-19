use crate::{
    config::{BASE_TOKEN_ID, MIN_PROFIT},
    models::graph::{Edge, Graph},
};
use multiversx_sc::{
    imports::{Bech32Address, MultiValue2},
    types::{ManagedAddress, MultiValueEncoded, TokenIdentifier},
};
use multiversx_sc_scenario::api::StaticApi;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use std::collections::{HashMap, HashSet};

pub fn swap(amount_in: &BigUint, edge: &Edge) -> BigUint {
    if edge.in_reserve.is_zero() || edge.out_reserve.is_zero() {
        return BigUint::zero();
    }

    let amount_in_after_fee = amount_in * (100_000u32 - edge.fee) / 100_000u32;
    let numerator = &amount_in_after_fee * &edge.out_reserve;
    let denominator = &edge.in_reserve + &amount_in_after_fee;
    numerator / denominator
}

pub fn simulate_triangle(amount_in: &BigUint, e1: &Edge, e2: &Edge, e3: &Edge) -> BigUint {
    let amount_after_e1 = swap(amount_in, e1);
    let amount_after_e2 = swap(&amount_after_e1, e2);
    if &swap(&amount_after_e2, e3) > amount_in {
        swap(&amount_after_e2, e3) - amount_in
    } else {
        BigUint::zero()
    }
}

pub type TradePath = MultiValueEncoded<
    StaticApi,
    MultiValue2<ManagedAddress<StaticApi>, TokenIdentifier<StaticApi>>,
>;

#[derive(Debug, Clone)]
pub struct TradeCandidate {
    pub profit: BigUint,
    pub token2_id: String,
    pub token3_id: String,
    pub sc_address1: String,
    pub sc_address2: String,
    pub sc_address3: String,
    pub amount_in: BigUint,
}

pub fn find_trade_path(graph: &Graph, amount_in: BigUint) -> Option<TradeCandidate> {
    let token1 = BASE_TOKEN_ID.to_string();

    let binding = Vec::new();
    let edges1 = graph.get(&token1).unwrap_or(&binding);

    let mut best_trade: Option<TradeCandidate> = None;

    for edge1 in edges1 {
        let token2 = &edge1.out_id;
        if let Some(edges2) = graph.get(token2) {
            for edge2 in edges2 {
                let token3 = &edge2.out_id;
                if token3 == &token1 {
                    continue;
                }
                if let Some(edges3) = graph.get(token3) {
                    for edge3 in edges3 {
                        if edge3.out_id == *token1 {
                            let profit = simulate_triangle(&amount_in, edge1, edge2, edge3);
                            if profit > BigUint::zero() {
                                let is_better_than_current = best_trade
                                    .as_ref()
                                    .map(|trade| profit > trade.profit)
                                    .unwrap_or(true);
                                if profit > MIN_PROFIT.into() && is_better_than_current {
                                    best_trade = Some(TradeCandidate {
                                        profit: profit.clone(),
                                        token2_id: token2.clone(),
                                        token3_id: token3.clone(),
                                        sc_address1: edge1.sc_address.clone(),
                                        sc_address2: edge2.sc_address.clone(),
                                        sc_address3: edge3.sc_address.clone(),
                                        amount_in: amount_in.clone(),
                                    });

                                    println!(
                                        "Found profitable path: {} -> {} -> {} with profit {}",
                                        token1, token2, token3, profit
                                    );
                                }
                                // println!(
                                //     "Found profitable path: {} -> {} -> {} with profit {}",
                                //     token1, token2, token3, profit
                                // );
                            }
                        }
                    }
                }
            }
        }
    }

    best_trade
}

pub fn build_trade_path(trade: &TradeCandidate) -> TradePath {
    let token1 = BASE_TOKEN_ID.to_string();
    let mut swaps: TradePath = MultiValueEncoded::new();

    swaps.push(MultiValue2::from((
        ManagedAddress::from_address(
            &Bech32Address::from_bech32_string(trade.sc_address1.clone()).address,
        ),
        TokenIdentifier::from_esdt_bytes(&trade.token2_id),
    )));

    swaps.push(MultiValue2::from((
        ManagedAddress::from_address(
            &Bech32Address::from_bech32_string(trade.sc_address2.clone()).address,
        ),
        TokenIdentifier::from_esdt_bytes(&trade.token3_id),
    )));

    swaps.push(MultiValue2::from((
        ManagedAddress::from_address(
            &Bech32Address::from_bech32_string(trade.sc_address3.clone()).address,
        ),
        TokenIdentifier::from_esdt_bytes(token1),
    )));

    swaps
}

#[derive(Debug, Clone)]
pub struct BellmanFordEdge {
    pub from_token: String,
    pub to_token: String,
    pub sc_address: String,
    pub effective_rate: f64,
    pub weight: f64,
}

#[derive(Debug, Clone)]
pub struct BellmanFordCycleCandidate {
    pub token_cycle: Vec<String>,
    pub pool_addresses: Vec<String>,
    pub estimated_multiplier: f64,
    pub amount_in: BigUint,
    pub estimated_amount_out: BigUint,
    pub estimated_profit: BigUint,
}

#[derive(Clone)]
struct BfIndexedEdge {
    from: usize,
    to: usize,
    pool_address: String,
    rate: f64,
    weight: f64,
}

fn edge_effective_rate(edge: &Edge) -> Option<f64> {
    if edge.in_reserve.is_zero() || edge.out_reserve.is_zero() {
        return None;
    }

    let in_reserve = edge.in_reserve.to_f64()?;
    let out_reserve = edge.out_reserve.to_f64()?;
    if in_reserve <= 0.0 || out_reserve <= 0.0 {
        return None;
    }

    let fee_factor = (100_000u32.saturating_sub(edge.fee)) as f64 / 100_000.0;
    let rate = (out_reserve / in_reserve) * fee_factor;

    if rate.is_finite() && rate > 0.0 {
        Some(rate)
    } else {
        None
    }
}

pub fn build_bellman_ford_edges(graph: &Graph) -> Vec<BellmanFordEdge> {
    let mut edges = Vec::new();

    for (from_token, token_edges) in graph {
        for edge in token_edges {
            let Some(rate) = edge_effective_rate(edge) else {
                continue;
            };

            edges.push(BellmanFordEdge {
                from_token: from_token.clone(),
                to_token: edge.out_id.clone(),
                sc_address: edge.sc_address.clone(),
                effective_rate: rate,
                weight: -rate.ln(),
            });
        }
    }

    edges
}

pub fn find_trade_cycle_bellman_ford(
    graph: &Graph,
    base_token_id: &str,
    amount_in: &BigUint,
    max_hops: Option<usize>,
) -> Option<BellmanFordCycleCandidate> {
    let bf_edges = build_bellman_ford_edges(graph);
    if bf_edges.is_empty() {
        return None;
    }

    let mut tokens: HashSet<String> = HashSet::new();
    for token in graph.keys() {
        tokens.insert(token.clone());
    }
    for edge in &bf_edges {
        tokens.insert(edge.from_token.clone());
        tokens.insert(edge.to_token.clone());
    }
    if !tokens.contains(base_token_id) {
        return None;
    }

    let mut index_to_token: Vec<String> = tokens.into_iter().collect();
    index_to_token.sort();
    let token_to_index: HashMap<String, usize> = index_to_token
        .iter()
        .cloned()
        .enumerate()
        .map(|(idx, token)| (token, idx))
        .collect();

    let mut edges: Vec<BfIndexedEdge> = Vec::new();
    for edge in bf_edges {
        let Some(&from) = token_to_index.get(&edge.from_token) else {
            continue;
        };
        let Some(&to) = token_to_index.get(&edge.to_token) else {
            continue;
        };
        edges.push(BfIndexedEdge {
            from,
            to,
            pool_address: edge.sc_address,
            rate: edge.effective_rate,
            weight: edge.weight,
        });
    }
    if edges.is_empty() {
        return None;
    }

    let vertex_count = index_to_token.len();
    let base_idx = *token_to_index.get(base_token_id)?;

    let mut dist = vec![f64::INFINITY; vertex_count];
    let mut pred: Vec<Option<(usize, usize)>> = vec![None; vertex_count];
    dist[base_idx] = 0.0;

    let epsilon = 1e-12_f64;
    for _ in 0..vertex_count.saturating_sub(1) {
        let mut changed = false;
        for (edge_idx, edge) in edges.iter().enumerate() {
            if !dist[edge.from].is_finite() {
                continue;
            }

            let next = dist[edge.from] + edge.weight;
            if next + epsilon < dist[edge.to] {
                dist[edge.to] = next;
                pred[edge.to] = Some((edge.from, edge_idx));
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut relaxable_edges: Vec<(usize, usize)> = Vec::new();
    for (edge_idx, edge) in edges.iter().enumerate() {
        if !dist[edge.from].is_finite() {
            continue;
        }

        let next = dist[edge.from] + edge.weight;
        if next + epsilon < dist[edge.to] {
            relaxable_edges.push((edge_idx, edge.to));
        }
    }

    if relaxable_edges.is_empty() {
        return None;
    }

    let mut best_trade: Option<BellmanFordCycleCandidate> = None;
    let mut seen_cycles: HashSet<String> = HashSet::new();

    for (edge_idx, cycle_vertex) in relaxable_edges {
        let mut pred_for_cycle = pred.clone();
        pred_for_cycle[cycle_vertex] = Some((edges[edge_idx].from, edge_idx));

        let Some(candidate) = build_cycle_candidate_from_seed(
            graph,
            base_token_id,
            amount_in,
            max_hops,
            cycle_vertex,
            &pred_for_cycle,
            &edges,
            &index_to_token,
        ) else {
            continue;
        };

        let cycle_key = format!(
            "{}|{}",
            candidate.token_cycle.join("->"),
            candidate.pool_addresses.join("->")
        );
        if !seen_cycles.insert(cycle_key) {
            continue;
        }

        let is_better = best_trade
            .as_ref()
            .map(|trade| candidate.estimated_profit > trade.estimated_profit)
            .unwrap_or(true);
        if is_better {
            best_trade = Some(candidate);
        }
    }

    best_trade
}

fn simulate_path_amount_out(
    graph: &Graph,
    token_cycle: &[String],
    pool_addresses: &[String],
    amount_in: &BigUint,
) -> Option<BigUint> {
    if token_cycle.len() < 2 || token_cycle.len() != pool_addresses.len() + 1 {
        return None;
    }

    let mut amount = amount_in.clone();
    for hop in 0..pool_addresses.len() {
        let from_token = &token_cycle[hop];
        let to_token = &token_cycle[hop + 1];
        let pool_address = &pool_addresses[hop];

        let edge = graph
            .get(from_token)?
            .iter()
            .find(|edge| edge.sc_address == *pool_address && edge.out_id == *to_token)?;

        amount = swap(&amount, edge);
    }

    Some(amount)
}

fn build_cycle_candidate_from_seed(
    graph: &Graph,
    base_token_id: &str,
    amount_in: &BigUint,
    max_hops: Option<usize>,
    cycle_vertex: usize,
    pred: &[Option<(usize, usize)>],
    edges: &[BfIndexedEdge],
    index_to_token: &[String],
) -> Option<BellmanFordCycleCandidate> {
    let vertex_count = index_to_token.len();
    let mut cycle_vertex = cycle_vertex;
    for _ in 0..vertex_count {
        cycle_vertex = pred.get(cycle_vertex)?.as_ref()?.0;
    }

    let cycle_start = cycle_vertex;
    let mut vertices_rev: Vec<usize> = vec![cycle_start];
    let mut edge_indices_rev: Vec<usize> = Vec::new();
    let mut cursor = cycle_start;

    loop {
        let (prev, edge_idx) = pred.get(cursor)?.as_ref().copied()?;
        vertices_rev.push(prev);
        edge_indices_rev.push(edge_idx);
        cursor = prev;

        if cursor == cycle_start {
            break;
        }
        if vertices_rev.len() > vertex_count + 1 {
            return None;
        }
    }

    vertices_rev.reverse();
    edge_indices_rev.reverse();
    if edge_indices_rev.is_empty() {
        return None;
    }
    if let Some(limit) = max_hops {
        if edge_indices_rev.len() > limit {
            return None;
        }
    }

    let token_cycle: Vec<String> = vertices_rev
        .iter()
        .map(|idx| index_to_token[*idx].clone())
        .collect();
    let base_pos = token_cycle[..token_cycle.len().saturating_sub(1)]
        .iter()
        .position(|token| token == base_token_id)?;

    let unique_len = token_cycle.len().saturating_sub(1);
    let mut rotated_tokens = Vec::with_capacity(unique_len + 1);
    for i in 0..unique_len {
        rotated_tokens.push(token_cycle[(base_pos + i) % unique_len].clone());
    }
    rotated_tokens.push(base_token_id.to_string());

    let mut rotated_edge_indices = Vec::with_capacity(edge_indices_rev.len());
    for i in 0..edge_indices_rev.len() {
        rotated_edge_indices.push(edge_indices_rev[(base_pos + i) % edge_indices_rev.len()]);
    }

    let rotated_addresses: Vec<String> = rotated_edge_indices
        .iter()
        .map(|edge_idx| edges[*edge_idx].pool_address.clone())
        .collect();

    let estimated_multiplier = rotated_edge_indices
        .iter()
        .fold(1.0_f64, |acc, edge_idx| acc * edges[*edge_idx].rate);

    let estimated_amount_out =
        simulate_path_amount_out(graph, &rotated_tokens, &rotated_addresses, amount_in)?;
    let estimated_profit = if estimated_amount_out > *amount_in {
        &estimated_amount_out - amount_in
    } else {
        BigUint::zero()
    };
    if estimated_profit.is_zero() {
        return None;
    }

    Some(BellmanFordCycleCandidate {
        token_cycle: rotated_tokens,
        pool_addresses: rotated_addresses,
        estimated_multiplier,
        amount_in: amount_in.clone(),
        estimated_amount_out,
        estimated_profit,
    })
}

pub fn build_trade_path_from_cycle(
    token_cycle: &[String],
    pool_addresses: &[String],
) -> Option<TradePath> {
    if token_cycle.len() < 2 || token_cycle.len() != pool_addresses.len() + 1 {
        return None;
    }

    let mut swaps: TradePath = MultiValueEncoded::new();

    for hop in 0..pool_addresses.len() {
        swaps.push(MultiValue2::from((
            ManagedAddress::from_address(
                &Bech32Address::from_bech32_string(pool_addresses[hop].clone()).address,
            ),
            TokenIdentifier::from_esdt_bytes(token_cycle[hop + 1].clone()),
        )));
    }

    Some(swaps)
}

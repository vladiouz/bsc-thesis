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
use num_traits::Zero;

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

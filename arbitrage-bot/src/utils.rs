use crate::{
    config::{AMOUNT_IN, BASE_TOKEN_ID, MIN_PROFIT},
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

pub fn find_trade_path(graph: &Graph) -> Option<TradePath> {
    let token1 = BASE_TOKEN_ID.to_string();
    let amount_in = BigUint::from(AMOUNT_IN);

    let binding = Vec::new();
    let edges1 = graph.get(&token1).unwrap_or(&binding);

    let mut token2_id = String::new();
    let mut token3_id = String::new();

    let mut max_profit = BigUint::zero();

    let mut sc_address1 = String::new();
    let mut sc_address2 = String::new();
    let mut sc_address3 = String::new();

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
                                if profit > MIN_PROFIT.into() && profit > max_profit {
                                    max_profit = profit.clone();
                                    token2_id = token2.clone();
                                    token3_id = token3.clone();
                                    sc_address1 = edge1.sc_address.clone();
                                    sc_address2 = edge2.sc_address.clone();
                                    sc_address3 = edge3.sc_address.clone();

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

    if max_profit > BigUint::zero() {
        let mut swaps: MultiValueEncoded<
            StaticApi,
            MultiValue2<ManagedAddress<StaticApi>, TokenIdentifier<StaticApi>>,
        > = MultiValueEncoded::new();

        swaps.push(MultiValue2::from((
            ManagedAddress::from_address(&Bech32Address::from_bech32_string(sc_address1).address),
            TokenIdentifier::from_esdt_bytes(token2_id),
        )));

        swaps.push(MultiValue2::from((
            ManagedAddress::from_address(&Bech32Address::from_bech32_string(sc_address2).address),
            TokenIdentifier::from_esdt_bytes(token3_id),
        )));

        swaps.push(MultiValue2::from((
            ManagedAddress::from_address(&Bech32Address::from_bech32_string(sc_address3).address),
            TokenIdentifier::from_esdt_bytes(token1),
        )));

        Some(swaps)
    } else {
        None
    }
}

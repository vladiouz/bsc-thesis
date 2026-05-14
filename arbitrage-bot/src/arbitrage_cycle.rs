use crate::api::pools_data::get_token_reserve_v2;
use crate::config::*;
use crate::metrics::log_metric;
use crate::models::graph::build_graph;
use crate::models::liquidity_pool::LiquidityPool;
use crate::utils::{build_trade_path, find_trade_path};
use arbitrage_interactor::interact;
use futures::future::join_all;
use rayon::prelude::*;
use reqwest::Client;
use std::time::Instant;

pub async fn run_arbitrage_cycle(client: &Client, liquidity_pools: &mut Vec<LiquidityPool>) {
    let fetching_reserves_timer = Instant::now();

    let reserve_futures = liquidity_pools
        .iter()
        .map(|lp| get_token_reserve_v2(client, &lp.sc_address));

    let reserves = join_all(reserve_futures).await;

    for (lp, (first_reserve, second_reserve)) in liquidity_pools.iter_mut().zip(reserves) {
        if lp.base_is_first {
            lp.base_reserve = first_reserve;
            lp.quote_reserve = second_reserve;
        } else {
            lp.base_reserve = second_reserve;
            lp.quote_reserve = first_reserve;
        }
    }

    // log_metric(
    //     VERSION,
    //     "fetching_reserves",
    //     fetching_reserves_timer.elapsed().as_micros(),
    // );

    let graph_building_timer = Instant::now();
    let graph = build_graph(liquidity_pools);
    // log_metric(
    //     VERSION,
    //     "graph_building",
    //     graph_building_timer.elapsed().as_micros(),
    // );

    let swaps_finding_timer = Instant::now();

    let results: Vec<_> = AMOUNTS_IN
        .par_iter()
        .map(|amount_in| find_trade_path(&graph, (*amount_in).into()))
        .flatten()
        .collect();

    let best_result = results.into_iter().max_by(|a, b| a.profit.cmp(&b.profit));

    // log_metric(
    //     VERSION,
    //     "swaps_finding",
    //     swaps_finding_timer.elapsed().as_micros(),
    // );

    match best_result {
        Some(trade) => {
            let swaps = build_trade_path(&trade);
            let mut interact = interact::ContractInteract::new().await;
            interact.execute_trades(trade.amount_in.into(), swaps).await;
            // println!("Arbitrage path found: {:?}", swaps);
        }
        // None => println!("No arbitrage path found"),
        None => {} // do nothing
    }
}

use crate::api::pools_data::get_token_reserve_v2;
// use crate::config::VERSION;
use crate::config::{AMOUNTS_IN, BASE_TOKEN_ID, MIN_PROFIT};
// use crate::metrics::log_metric;
use crate::models::graph::build_graph;
use crate::models::liquidity_pool::LiquidityPool;
use crate::utils::{
    build_trade_path, build_trade_path_from_cycle, find_trade_cycle_bellman_ford, find_trade_path,
};
use arbitrage_interactor::interact;
use futures::future::join_all;
use rayon::prelude::*;
use reqwest::Client;
// use std::time::Instant;

async fn refresh_pool_reserves(client: &Client, liquidity_pools: &mut [LiquidityPool]) {
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
}

pub async fn run_triangle_arbitrage_cycle(
    client: &Client,
    liquidity_pools: &mut Vec<LiquidityPool>,
) {
    // let fetching_reserves_timer = Instant::now();
    refresh_pool_reserves(client, liquidity_pools).await;
    // log_metric(
    //     VERSION,
    //     "fetching_reserves",
    //     fetching_reserves_timer.elapsed().as_micros(),
    // );

    // let graph_building_timer = Instant::now();
    let graph = build_graph(liquidity_pools);
    // log_metric(
    //     VERSION,
    //     "graph_building",
    //     graph_building_timer.elapsed().as_micros(),
    // );

    // let swaps_finding_timer = Instant::now();
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

    if let Some(trade) = best_result {
        let swaps = build_trade_path(&trade);
        let mut interact = interact::ContractInteract::new().await;
        interact.execute_trades(trade.amount_in.into(), swaps).await;
    }
}

pub async fn run_bellman_ford_arbitrage_cycle(
    client: &Client,
    liquidity_pools: &mut Vec<LiquidityPool>,
) {
    // let fetching_reserves_timer = Instant::now();
    refresh_pool_reserves(client, liquidity_pools).await;
    // log_metric(
    //     VERSION,
    //     "fetching_reserves",
    //     fetching_reserves_timer.elapsed().as_micros(),
    // );

    // let graph_building_timer = Instant::now();
    let graph = build_graph(liquidity_pools);
    // log_metric(
    //     VERSION,
    //     "graph_building",
    //     graph_building_timer.elapsed().as_micros(),
    // );

    // let swaps_finding_timer = Instant::now();
    let results: Vec<_> = AMOUNTS_IN
        .par_iter()
        .map(|amount_in| {
            let amount = (*amount_in).into();
            find_trade_cycle_bellman_ford(&graph, BASE_TOKEN_ID, &amount, None)
        })
        .flatten()
        .collect();
    // log_metric(
    //     VERSION,
    //     "swaps_finding",
    //     swaps_finding_timer.elapsed().as_micros(),
    // );

    let best_result = results
        .into_iter()
        .max_by(|a, b| a.estimated_profit.cmp(&b.estimated_profit));

    if let Some(trade) = best_result {
        if trade.estimated_profit <= MIN_PROFIT.into() {
            return;
        }

        if let Some(swaps) = build_trade_path_from_cycle(&trade.token_cycle, &trade.pool_addresses)
        {
            println!(
                "Chosen Bellman-Ford path: {} | pools: {} | amount_in: {} | est_amount_out: {} | est_profit: {} | est_multiplier: {:.8}",
                trade.token_cycle.join(" -> "),
                trade.pool_addresses.join(" -> "),
                trade.amount_in,
                trade.estimated_amount_out,
                trade.estimated_profit,
                trade.estimated_multiplier
            );

            let mut interact = interact::ContractInteract::new().await;
            interact.execute_trades(trade.amount_in.into(), swaps).await;
        }
    }
}

pub async fn run_arbitrage_cycle(client: &Client, liquidity_pools: &mut Vec<LiquidityPool>) {
    run_triangle_arbitrage_cycle(client, liquidity_pools).await;
}

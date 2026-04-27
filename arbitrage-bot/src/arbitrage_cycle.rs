use crate::api::pools_data::get_token_reserve;
use crate::config::*;
use crate::metrics::log_metric;
use crate::models::graph::build_graph;
use crate::models::liquidity_pool::LiquidityPool;
use crate::utils::find_trade_path;
use arbitrage_interactor::interact;
use reqwest::Client;
use std::time::Instant;

pub async fn run_arbitrage_cycle(client: &Client, liquidity_pools: &mut Vec<LiquidityPool>) {
    let fetching_reserves_timer = Instant::now();

    for lp in liquidity_pools.iter_mut() {
        lp.base_reserve = get_token_reserve(client, &lp.sc_address, &lp.base_id).await;
        lp.quote_reserve = get_token_reserve(client, &lp.sc_address, &lp.quote_id).await;
    }

    log_metric(
        VERSION,
        "fetching_reserves",
        fetching_reserves_timer.elapsed().as_micros(),
    );

    let graph_building_timer = Instant::now();
    let graph = build_graph(liquidity_pools);
    log_metric(
        VERSION,
        "graph_building",
        graph_building_timer.elapsed().as_micros(),
    );

    let swaps_finding_timer = Instant::now();
    let swaps_option = find_trade_path(&graph);
    log_metric(
        VERSION,
        "swaps_finding",
        swaps_finding_timer.elapsed().as_micros(),
    );

    match swaps_option {
        Some(swaps) => {
            let mut interact = interact::ContractInteract::new().await;
            interact.execute_trades(swaps).await;
        }
        None => println!("No arbitrage path found"),
    }
}

pub mod api;
pub mod config;
pub mod metrics;
pub mod models;
pub mod utils;

use crate::api::all_pools::fetch_all_pools;
use crate::api::pools_data::{get_fee, get_token_reserve};
use crate::metrics::log_metric;
use crate::models::graph::build_graph;
use crate::models::liquidity_pool::{LiquidityPool, filter_pools};
use crate::utils::find_trade_path;
use arbitrage_interactor::interact;
use config::*;
use reqwest::Client;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let pools_fetching_timer = Instant::now();

    let res_json = fetch_all_pools(&client).await;
    log_metric(
        VERSION,
        "pools_fetching",
        pools_fetching_timer.elapsed().as_micros(),
    );

    let mut liquidity_pools: Vec<LiquidityPool> = Vec::new();
    let lps_creation_timer = Instant::now();

    for pair in res_json.as_array().unwrap() {
        let sc_address = &pair["address"];
        let base_id = &pair["baseId"];
        let quote_id = &pair["quoteId"];

        liquidity_pools.push(LiquidityPool::new(
            sc_address.as_str().unwrap().to_string(),
            base_id.as_str().unwrap().to_string(),
            quote_id.as_str().unwrap().to_string(),
        ));
    }
    log_metric(
        VERSION,
        "lps_creation",
        lps_creation_timer.elapsed().as_micros(),
    );

    // /mex/pairs endpoint might be down and return []
    if liquidity_pools.is_empty() {
        let file = File::open("pairs.csv").unwrap();
        let reader = BufReader::new(file);
        let lps_creation_backup_timer = Instant::now();

        for line in reader.lines().skip(1) {
            let line = line.unwrap();
            let parts: Vec<&str> = line.split(',').collect();

            let address = parts[0];
            let token1 = parts[1];
            let token2 = parts[2];

            liquidity_pools.push(LiquidityPool::new(
                address.to_string(),
                token1.to_string(),
                token2.to_string(),
            ));
        }
        log_metric(
            VERSION,
            "lps_creation_backup",
            lps_creation_backup_timer.elapsed().as_micros(),
        );
    }

    let filtering_timer = Instant::now();
    liquidity_pools = filter_pools(liquidity_pools, BASE_TOKEN_ID);
    log_metric(
        VERSION,
        "pools_filtering",
        filtering_timer.elapsed().as_micros(),
    );

    let fetching_reserves_timer = Instant::now();

    for lp in &mut liquidity_pools {
        lp.fee = get_fee(&client, &lp.sc_address).await;
        lp.base_reserve = get_token_reserve(&client, &lp.sc_address, &lp.base_id).await;
        lp.quote_reserve = get_token_reserve(&client, &lp.sc_address, &lp.quote_id).await;
    }

    log_metric(
        VERSION,
        "fetching_reserves",
        fetching_reserves_timer.elapsed().as_micros(),
    );

    let graph_building_timer = Instant::now();
    let graph = build_graph(&liquidity_pools);

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

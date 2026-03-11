pub mod api;
pub mod config;
pub mod models;
pub mod utils;

use crate::api::all_pools::fetch_all_pools;
use crate::api::pools_data::{get_fee, get_token_reserve};
use crate::models::graph::build_graph;
use crate::models::liquidity_pool::LiquidityPool;
use crate::utils::find_trade_path;
use arbitrage_interactor::interact;
use config::*;
use reqwest::Client;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[tokio::main]
async fn main() {
    let client = Client::new();

    let res_json = fetch_all_pools(&client).await;

    let mut liquidity_pools: Vec<LiquidityPool> = Vec::new();

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

    // /mex/pairs endpoint might be down and return []
    if liquidity_pools.is_empty() {
        let file = File::open("pairs.csv").unwrap();
        let reader = BufReader::new(file);

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
    }

    println!("Liquidity pools count: {}", liquidity_pools.len());

    for lp in &mut liquidity_pools {
        lp.fee = get_fee(&client, &lp.sc_address).await;
        lp.base_reserve = get_token_reserve(&client, &lp.sc_address, &lp.base_id).await;
        lp.quote_reserve = get_token_reserve(&client, &lp.sc_address, &lp.quote_id).await;
    }

    let graph = build_graph(&liquidity_pools);
    println!("Graph size: {}", graph.len());

    let swaps_option = find_trade_path(&graph);
    match swaps_option {
        Some(swaps) => {
            let mut interact = interact::ContractInteract::new().await;
            interact.execute_trades(swaps).await;
        }
        None => println!("No arbitrage path found"),
    }
}

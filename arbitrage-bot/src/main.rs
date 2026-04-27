pub mod api;
pub mod arbitrage_cycle;
pub mod config;
pub mod metrics;
pub mod models;
pub mod rabbitmq_consumer;
pub mod utils;

use crate::api::all_pools::fetch_all_pools;
use crate::api::pools_data::get_fee;
use crate::arbitrage_cycle::run_arbitrage_cycle;
use crate::metrics::log_metric;
use crate::models::liquidity_pool::{LiquidityPool, filter_pools};
use crate::rabbitmq_consumer::setup_consumer;
use config::*;
use reqwest::Client;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;
use tokio::sync::mpsc;

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

    if liquidity_pools.is_empty() {
        let file = File::open("pairs.csv").unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line.unwrap();
            let parts: Vec<&str> = line.split(',').collect();
            liquidity_pools.push(LiquidityPool::new(
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ));
        }
    }

    liquidity_pools = filter_pools(liquidity_pools, BASE_TOKEN_ID);

    for lp in &mut liquidity_pools {
        lp.fee = get_fee(&client, &lp.sc_address).await;
    }

    let (trigger_sender, mut trigger_receiver) = mpsc::unbounded_channel::<()>();

    let client_clone = client.clone();
    let mut pools_clone = liquidity_pools.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            while trigger_receiver.recv().await.is_some() {
                run_arbitrage_cycle(&client_clone, &mut pools_clone).await;
            }
        });
    });

    setup_consumer(trigger_sender).await.unwrap();

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

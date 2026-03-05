use hex;
use multiversx_sdk::{
    crypto::SigningKeyPair,
    network::{providers::HttpNetworkProvider, transactions::Transaction},
};
use num_bigint::BigUint;
use num_traits::Zero;
use reqwest::Client;
use serde_json::Value;
use serde_json::from_str;
use std::collections::HashMap;

const BASE_API: &str = "https://devnet-api.multiversx.com";
const BASE_GATEWAY: &str = "https://devnet-gateway.multiversx.com";

struct LiquidityPool {
    sc_address: String,
    base_id: String,
    quote_id: String,
    base_reserve: BigUint,
    quote_reserve: BigUint,
    fee: u32,
}

impl LiquidityPool {
    pub fn new(sc_address: String, base_id: String, quote_id: String) -> Self {
        LiquidityPool {
            sc_address,
            base_id,
            quote_id,
            base_reserve: Zero::zero(),
            quote_reserve: Zero::zero(),
            fee: 1000,
        }
    }
}

#[derive(Debug)]
struct Edge {
    in_id: String,
    out_id: String,
    in_reserve: BigUint,
    out_reserve: BigUint,
    fee: u32,
}

type Graph = HashMap<String, Vec<Edge>>;

fn build_graph(pools: &Vec<LiquidityPool>) -> Graph {
    let mut graph: Graph = HashMap::new();

    for pool in pools {
        graph.entry(pool.base_id.clone()).or_default().push(Edge {
            in_id: pool.base_id.clone(),
            out_id: pool.quote_id.clone(),
            in_reserve: pool.base_reserve.clone(),
            out_reserve: pool.quote_reserve.clone(),
            fee: pool.fee,
        });

        graph.entry(pool.quote_id.clone()).or_default().push(Edge {
            in_id: pool.quote_id.clone(),
            out_id: pool.base_id.clone(),
            in_reserve: pool.quote_reserve.clone(),
            out_reserve: pool.base_reserve.clone(),
            fee: pool.fee,
        });
    }

    graph
}

fn swap(amount_in: &BigUint, edge: &Edge) -> BigUint {
    let amount_in_after_fee = amount_in * (1_000_000u32 - edge.fee) / 1_000_000u32;
    let numerator = &amount_in_after_fee * &edge.out_reserve;
    let denominator = &edge.in_reserve + &amount_in_after_fee;
    numerator / denominator
}

fn simulate_triangle(amount_in: &BigUint, e1: &Edge, e2: &Edge, e3: &Edge) -> BigUint {
    let amount_after_e1 = swap(amount_in, e1);
    let amount_after_e2 = swap(&amount_after_e1, e2);
    if &swap(&amount_after_e2, e3) > amount_in {
        swap(&amount_after_e2, e3) - amount_in
    } else {
        BigUint::zero()
    }
}

#[tokio::main]
async fn main() {
    let client = Client::new();

    let response = client
        .get(format!(
            "{}/mex/pairs?size=500&exchange=xexchange&includeFarms=false",
            BASE_API
        ))
        .send()
        .await;

    let mut res_json = Value::Null;

    match response {
        Ok(resp) => match resp.text().await {
            Ok(text) => res_json = from_str(&text).unwrap(),
            Err(e) => eprintln!("Body error: {}", e),
        },
        Err(e) => eprintln!("Request error: {}", e),
    }

    // println!("{}", res_json);
    // println!("{}", res_json.as_array().unwrap().len());
    let mut tokens: HashMap<&Value, bool> = HashMap::new();

    let mut liquidity_pools: Vec<LiquidityPool> = Vec::new();

    for pair in res_json.as_array().unwrap() {
        let sc_address = &pair["address"];
        // println!("SC address: {}", sc_address);

        // let base_price = &pair["basePrice"];
        // let quote_price = &pair["quotePrice"];
        let base_id = &pair["baseId"];
        let quote_id = &pair["quoteId"];

        liquidity_pools.push(LiquidityPool::new(
            sc_address.as_str().unwrap().to_string(),
            base_id.as_str().unwrap().to_string(),
            quote_id.as_str().unwrap().to_string(),
        ));

        // check it either one starts with ITHEUM

        // if base_id.as_str().unwrap().starts_with("ITHEUM")
        //     || quote_id.as_str().unwrap().starts_with("ITHEUM")
        // {
        //     println!("Found ITHEUM pair: {}", sc_address);
        //     break;
        // }

        // println!("Base Price: {}, Quote Price: {}", base_price, quote_price);
        // println!("Base ID: {}, Quote ID: {}", base_id, quote_id);

        if tokens.contains_key(base_id) {
            // println!("Token {} already exists", base_id);
        } else {
            tokens.insert(base_id, true);
        }

        if tokens.contains_key(quote_id) {
            // println!("Token {} already exists", quote_id);
        } else {
            tokens.insert(quote_id, true);
        }

        // let price = base_price.as_f64().unwrap() / quote_price.as_f64().unwrap();
        // println!("Price: {}", price);
    }

    println!("Total unique tokens: {}", tokens.len());

    for lp in &mut liquidity_pools {
        let fee_response = client
            .post(format!("{}/vm-values/int", BASE_GATEWAY))
            .json(&serde_json::json!({
                "scAddress": lp.sc_address,
                "funcName": "getTotalFeePercent"
            }))
            .send()
            .await;

        let mut fee_percent: Value = Value::Number(1000.into());

        match fee_response {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    let fee_json: Value = from_str(&text).unwrap();
                    fee_percent = fee_json.as_object().unwrap()["data"]["data"].clone();
                    // println!(
                    //     "LP Address: {}, Total Fee Percent: {}",
                    //     lp.sc_address, fee_percent
                    // );
                }
                Err(e) => eprintln!("Body error: {}", e),
            },
            Err(e) => eprintln!("Request error: {}", e),
        }

        let base_token_response = client
            .post(format!("{}/vm-values/int", BASE_GATEWAY))
            .json(&serde_json::json!({
                "scAddress": lp.sc_address,
                "funcName": "getReserve",
                "args": [hex::encode(&lp.base_id)]
            }))
            .send()
            .await;

        let mut base_token_reserve: Value = Value::Null;

        match base_token_response {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    let base_token_json: Value = from_str(&text).unwrap();
                    base_token_reserve =
                        base_token_json.as_object().unwrap()["data"]["data"].clone();
                    // println!(
                    //     "LP Address: {}, Base Token Reserve: {}",
                    //     lp.sc_address, base_token_reserve
                    // );
                }
                Err(e) => eprintln!("Body error: {}", e),
            },
            Err(e) => eprintln!("Request error: {}", e),
        }

        let mut quote_token_reserve: Value = Value::Null;

        let quote_token_response = client
            .post(format!("{}/vm-values/int", BASE_GATEWAY))
            .json(&serde_json::json!({
                "scAddress": lp.sc_address,
                "funcName": "getReserve",
                "args": [hex::encode(&lp.quote_id)]
            }))
            .send()
            .await;

        match quote_token_response {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    let quote_token_json: Value = from_str(&text).unwrap();
                    quote_token_reserve =
                        quote_token_json.as_object().unwrap()["data"]["data"].clone();
                    // println!(
                    //     "LP Address: {}, Quote Token Reserve: {}",
                    //     lp.sc_address, quote_token_reserve
                    // );
                }
                Err(e) => eprintln!("Body error: {}", e),
            },
            Err(e) => eprintln!("Request error: {}", e),
        }

        // // calculate price
        // if base_token_reserve.as_str().unwrap() != "0"
        //     && quote_token_reserve.as_str().unwrap() != "0"
        // {
        //     let price = base_token_reserve.as_str().unwrap().parse::<f64>().unwrap()
        //         / quote_token_reserve
        //             .as_str()
        //             .unwrap()
        //             .parse::<f64>()
        //             .unwrap();
        //     println!("LP Address: {}, Price: {}", lp.sc_address, price);
        // } else {
        //     println!(
        //         "LP Address: {}, Price: Cannot calculate due to zero reserve",
        //         lp.sc_address
        //     );
        // }

        lp.base_reserve = base_token_reserve
            .as_str()
            .and_then(|s| s.parse::<BigUint>().ok())
            .unwrap_or_else(BigUint::zero);

        lp.quote_reserve = quote_token_reserve
            .as_str()
            .and_then(|s| s.parse::<BigUint>().ok())
            .unwrap_or_else(BigUint::zero);

        lp.fee = fee_percent
            .as_str()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1000);
    }

    let graph = build_graph(&liquidity_pools);
    println!("Graph len: {:#?}", graph.len());

    let token1: String = "USDC-350c4e".to_string();
    let amount_in = BigUint::from(1_000_000u32);
    let binding = Vec::new();
    let edges1 = graph.get(&token1).unwrap_or(&binding);

    let mut token2_id = String::new();
    let mut token3_id = String::new();
    let mut max_profit = BigUint::zero();

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
                                println!(
                                    "Arbitrage opportunity: {} -> {} -> {} -> {} | Profit: {}",
                                    token1, token2, token3, token1, profit
                                );
                                if profit > max_profit {
                                    max_profit = profit;
                                    token2_id = token2.clone();
                                    token3_id = token3.clone();
                                }
                            } else {
                                println!(
                                    "No arbitrage: {} -> {} -> {} -> {} | Profit: {}",
                                    token1, token2, token3, token1, profit
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    if max_profit > BigUint::zero() {
        println!(
            "Best arbitrage path: {} -> {} -> {} -> {} | Max Profit: {}",
            token1, token2_id, token3_id, token1, max_profit
        );
    } else {
        println!("No arbitrage opportunities found for {}", token1);
    }
}

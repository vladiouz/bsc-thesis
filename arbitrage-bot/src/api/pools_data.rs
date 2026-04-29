use crate::config::{BASE_GATEWAY, DEFAULT_FEE};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use num_bigint::BigUint;
use num_traits::Zero;
use reqwest::Client;
use serde_json::{Value, from_str};

pub async fn get_fee(client: &Client, lp_address: &String) -> u32 {
    let response = client
        .post(format!("{}/vm-values/int", BASE_GATEWAY))
        .json(&serde_json::json!({
            "scAddress": lp_address,
            "funcName": "getTotalFeePercent"
        }))
        .send()
        .await;

    let mut fee: Value = Value::Number(DEFAULT_FEE.into());

    match response {
        Ok(resp) => match resp.text().await {
            Ok(text) => match from_str::<Value>(&text) {
                Ok(fee_json) => {
                    fee = fee_json
                        .get("data")
                        .and_then(|v| v.get("data"))
                        .cloned()
                        .unwrap_or_else(|| Value::Number(DEFAULT_FEE.into()));
                }
                Err(e) => eprintln!("Failed to parse fee JSON for {}: {}", lp_address, e),
            },
            Err(e) => eprintln!("Body error: {}", e),
        },
        Err(e) => eprintln!("Request error: {}", e),
    }

    fee.as_str()
        .and_then(|fee_str| fee_str.parse::<u32>().ok())
        .unwrap_or(DEFAULT_FEE)
}

pub async fn get_first_token_id(client: &Client, lp_address: &String) -> Option<String> {
    let response = client
        .post(format!("{}/vm-values/string", BASE_GATEWAY))
        .json(&serde_json::json!({
            "scAddress": lp_address,
            "funcName": "getFirstTokenId"
        }))
        .send()
        .await;

    match response {
        Ok(resp) => match resp.text().await {
            Ok(text) => match from_str::<Value>(&text) {
                Ok(token_json) => token_json
                    .get("data")
                    .and_then(|v| v.get("data"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
                Err(e) => {
                    eprintln!("Failed to parse first token JSON for {}: {}", lp_address, e);
                    None
                }
            },
            Err(e) => {
                eprintln!("Body error: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("Request error: {}", e);
            None
        }
    }
}

pub async fn get_token_reserve(client: &Client, lp_address: &String, token_id: &String) -> BigUint {
    let response = client
        .post(format!("{}/vm-values/int", BASE_GATEWAY))
        .json(&serde_json::json!({
            "scAddress": lp_address,
            "funcName": "getReserve",
            "args": [hex::encode(token_id)]
        }))
        .send()
        .await;

    let mut reserve: Value = Value::Null;

    match response {
        Ok(resp) => match resp.text().await {
            Ok(text) => match from_str::<Value>(&text) {
                Ok(reserve_json) => {
                    reserve = reserve_json
                        .get("data")
                        .and_then(|v| v.get("data"))
                        .cloned()
                        .unwrap_or(Value::Null);
                }
                Err(e) => {
                    eprintln!(
                        "Failed to parse reserve JSON for {} / {}: {}",
                        lp_address, token_id, e
                    )
                }
            },
            Err(e) => eprintln!("Body error: {}", e),
        },
        Err(e) => eprintln!("Request error: {}", e),
    }

    reserve
        .as_str()
        .and_then(|s| s.parse::<BigUint>().ok())
        .unwrap_or_else(BigUint::zero)
}

pub async fn get_token_reserve_v2(client: &Client, lp_address: &String) -> (BigUint, BigUint) {
    let response = client
        .post(format!("{}/vm-values/query", BASE_GATEWAY))
        .json(&serde_json::json!({
            "scAddress": lp_address,
            "funcName": "getReservesAndTotalSupply"
        }))
        .send()
        .await;

    let mut reserves = (BigUint::zero(), BigUint::zero());

    match response {
        Ok(resp) => match resp.text().await {
            Ok(text) => match from_str::<Value>(&text) {
                Ok(reserves_json) => {
                    let return_data = reserves_json
                        .get("data")
                        .and_then(|v| v.get("data"))
                        .and_then(|v| v.get("returnData"))
                        .and_then(Value::as_array)
                        .or_else(|| {
                            reserves_json
                                .get("data")
                                .and_then(|v| v.get("data"))
                                .and_then(Value::as_array)
                        })
                        .or_else(|| reserves_json.as_array());

                    if let Some(values) = return_data {
                        let base = values
                            .first()
                            .and_then(Value::as_str)
                            .and_then(|encoded| STANDARD.decode(encoded).ok())
                            .map(|bytes| BigUint::from_bytes_be(&bytes))
                            .unwrap_or_else(BigUint::zero);

                        let quote = values
                            .get(1)
                            .and_then(Value::as_str)
                            .and_then(|encoded| STANDARD.decode(encoded).ok())
                            .map(|bytes| BigUint::from_bytes_be(&bytes))
                            .unwrap_or_else(BigUint::zero);

                        reserves = (base, quote);
                    } else {
                        eprintln!(
                            "Missing returnData array while parsing reserves for {}",
                            lp_address
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse reserves v2 JSON for {}: {}", lp_address, e)
                }
            },
            Err(e) => eprintln!("Body error: {}", e),
        },
        Err(e) => eprintln!("Request error: {}", e),
    }

    reserves
}

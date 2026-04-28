use crate::config::{BASE_GATEWAY, DEFAULT_FEE};
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

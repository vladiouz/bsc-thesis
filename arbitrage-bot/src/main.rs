use reqwest::Client;
use serde_json::Value;
use serde_json::from_str;
use std::collections::HashMap;

const BASE_URL: &str = "https://devnet-api.multiversx.com";

#[tokio::main]
async fn main() {
    let client = Client::new();

    let response = client
        .get(format!(
            "{}/mex/pairs?size=500&exchange=xexchange&includeFarms=false",
            BASE_URL
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

    for pair in res_json.as_array().unwrap() {
        let sc_address = &pair["address"];
        println!("SC address: {}", sc_address);

        let base_price = &pair["basePrice"];
        let quote_price = &pair["quotePrice"];
        let base_id = &pair["baseId"];
        let quote_id = &pair["quoteId"];

        // check it either one starts with ITHEUM

        if base_id.as_str().unwrap().starts_with("ITHEUM")
            || quote_id.as_str().unwrap().starts_with("ITHEUM")
        {
            println!("Found ITHEUM pair: {}", sc_address);
            break;
        }

        println!("Base Price: {}, Quote Price: {}", base_price, quote_price);
        println!("Base ID: {}, Quote ID: {}", base_id, quote_id);

        if tokens.contains_key(base_id) {
            println!("Token {} already exists", base_id);
        } else {
            tokens.insert(base_id, true);
        }

        if tokens.contains_key(quote_id) {
            println!("Token {} already exists", quote_id);
        } else {
            tokens.insert(quote_id, true);
        }

        let price = base_price.as_f64().unwrap() / quote_price.as_f64().unwrap();
        println!("Price: {}", price);
    }

    println!("Total unique tokens: {}", tokens.len());
}

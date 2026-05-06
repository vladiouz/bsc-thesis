use crate::BASE_API;
use reqwest::Client;
use serde_json::{Value, from_str};

pub async fn fetch_all_pools(client: &Client) -> Value {
    let response = client
        .get(format!(
            "{}/mex/pairs?size=550&exchange=xexchange&includeFarms=false",
            BASE_API
        ))
        .send()
        .await
        .unwrap();

    let text = response.text().await.unwrap();
    from_str(&text).unwrap()
}

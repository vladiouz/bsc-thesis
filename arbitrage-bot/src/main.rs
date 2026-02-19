use reqwest::Client;

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

    match response {
        Ok(resp) => match resp.text().await {
            Ok(text) => println!("{}", text),
            Err(e) => eprintln!("Body error: {}", e),
        },
        Err(e) => eprintln!("Request error: {}", e),
    }
}

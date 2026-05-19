use arbitrage_bot::bot_runner::{BotMode, run_bot};

#[tokio::main]
async fn main() {
    run_bot(BotMode::Triangle).await;
}

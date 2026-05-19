use arbitrage_bot::bot_runner::{BotMode, run_bot};

fn parse_mode(arg: &str) -> Option<BotMode> {
    match arg {
        "triangle" => Some(BotMode::Triangle),
        "bellman-ford" => Some(BotMode::BellmanFord),
        "bellman_ford" => Some(BotMode::BellmanFord),
        _ => None,
    }
}

#[tokio::main]
async fn main() {
    let mode = match std::env::args().nth(1) {
        Some(arg) => match parse_mode(&arg) {
            Some(mode) => mode,
            None => {
                eprintln!("Unknown mode '{}'. Use 'triangle' or 'bellman-ford'.", arg);
                std::process::exit(2);
            }
        },
        None => BotMode::Triangle,
    };

    run_bot(mode).await;
}

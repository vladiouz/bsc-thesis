use std::fs::OpenOptions;
use std::io::Write;

pub fn log_metric(version: &str, stage: &str, duration_micro: u128) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("metrics.csv")
        .unwrap();

    writeln!(
        file,
        "{},{},{},{}",
        chrono::Utc::now(),
        version,
        stage,
        duration_micro
    )
    .unwrap();
}

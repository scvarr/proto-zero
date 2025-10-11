use serde::Serialize;
use core::evaluate_stdin_once;

#[derive(Serialize)]
struct Metrics<'a> {
    agent: &'a str,
    stage: &'a str,
    sensor: SensorMetrics,
}

#[derive(Serialize)]
struct SensorMetrics {
    has_any: bool,
}

fn main() {
    // Та же логика сенсора, но white имеет право логировать метрики.
    let has_any = evaluate_stdin_once().unwrap_or(false);

    let m = Metrics {
        agent: "white",
        stage: "Stage-0",
        sensor: SensorMetrics { has_any },
    };

    println!("{}", serde_json::to_string(&m).unwrap());
}

use serde::Serialize;
use protozero_core::{run_stdin_forver};

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
    // На каждое событие записываем метрику (одна JSON-строка).
    let _ = run_stdin_forver(|_chunk| {
        let m = Metrics {
            agent: "white",
            stage: "Stage-0",
            sensor: SensorMetrics { has_any: true },
        };
        println!("{}", serde_json::to_string(&m).unwrap());
    });    
}

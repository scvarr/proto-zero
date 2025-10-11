use serde::Serialize;
use protozero_core::{run_stdin_forver};

#[derive(Serialize)]
struct Metrics<'a> {
    agent: &'a str,
    stage: &'a str,
    sensor: SensorMetrics,
    drive: DriveMetrics,
}

#[derive(Serialize)]
struct DriveMetrics {
    #[serde(rename = "0")]
    d0: f64,
}

#[derive(Serialize)]
struct SensorMetrics {
    has_any: bool,
}

fn main() {
    // Внутреннее состояние драйва (эфемерно).
    let mut drive_0: f64 = 0.0;

    // На каждое событие записываем метрику (одна JSON-строка).
    let _ = run_stdin_forver(|_chunk| {
        drive_0 += 1.0;
        let m = Metrics {
            agent: "white",
            stage: "Stage-0",
            sensor: SensorMetrics { has_any: true },
            drive: DriveMetrics { d0: drive_0 },
        };
        println!("{}", serde_json::to_string(&m).unwrap());
    });
}

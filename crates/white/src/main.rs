//! WHITE: анатомический двойник BLACK, имеет право логирования метрик.
//! Stage-2 (ADR-011): d ← 1/(2−d); экспорт Prometheus на /metrics

mod metrics;

use std::net::SocketAddr;
use std::thread;
use std::time::Duration;
use metrics::{start_metrics_server, DRIVE0, EVENTS_TOTAL};

use serde::Serialize;
use protozero_core::{run_stdin_forever};
use protozero_core::drive::drive_update;

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
    // Запускаем HTTP-эндпоинт в отдельном треде (внешнее наблюдение; "время" внутрь не заходит).
    let addr: SocketAddr = std::env::var("WHITE_METRICS_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9100".into())
        .parse()
        .expect("bad addr");
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_metrics_server(addr));
    });

    // Внутреннее состояние драйва (эфемерно).
    let mut drive_0: f64 = 0.0;

    // На каждое событие записываем метрику (одна JSON-строка).
    let _ = run_stdin_forever(|_chunk| {
        drive_0 = drive_update(drive_0);

        // Обновляем экспортируемые метрики (тред-safe через атомики внутри Prometheus)
        DRIVE0.set(drive_0);
        EVENTS_TOTAL.inc();

        thread::sleep(Duration::from_millis(100));
        // (Опционально) печать JSON, можно отключить env-переменной
        if std::env::var("WHITE_LOG_JSON").ok().as_deref() != Some("0") {
            let m = Metrics {
                agent: "white",
                stage: "Stage-2",
                sensor: SensorMetrics { has_any: true },
                drive: DriveMetrics { d0: drive_0 },
            };
            println!("{}", serde_json::to_string(&m).unwrap());
        }
    });
}

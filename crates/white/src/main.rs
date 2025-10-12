//! WHITE: анатомический двойник BLACK, имеет право логирования метрик.
//! Stage-3 (ADR-014): публикуем d и Δ в Prometheus (+ опционально JSON).

mod metrics;

use std::net::SocketAddr;
use std::thread;
use metrics::{start_metrics_server, DRIVE0, EVENTS_TOTAL};

use serde::Serialize;
use protozero_core::{run_stdin_forever, DriveDiff};
use protozero_core::drive::{drive_update, FrameCounter};
use crate::metrics::{DRIVE0_DELTA, DRIVE0_FRAME, DRIVE0_FRAME_DELTA, FRAMES_TOTAL};

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
    delta: f64,
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

    let mut d_global = 0.0;
    let mut d_global_diff = DriveDiff::new();

    let mut frame = FrameCounter::new();
    let mut d_frame_prev: f64 = 0.0;

    let _ = run_stdin_forever(|chunk: &[u8]| {
        for &b in chunk {
            if b == b'\n' {
                frame.on_boundary();
                d_frame_prev = 0.0;
                continue;
            }

            // глобальный
            d_global = drive_update(d_global);
            let dg = d_global_diff.step(d_global);
            DRIVE0.set(d_global);
            DRIVE0_DELTA.set(dg);

            // покадровый
            let d_frame = frame.on_event();
            let df_delta = d_frame - d_frame_prev;
            d_frame_prev = d_frame;

            DRIVE0_FRAME.set(d_frame);
            DRIVE0_FRAME_DELTA.set(df_delta);

            EVENTS_TOTAL.inc();
        }
    });
}

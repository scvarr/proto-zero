//! WHITE: анатомический двойник BLACK, имеет право логирования метрик.
//! ADR-016: non-blocking сенсор — агент различает стимул и тишину.

mod metrics;

use metrics::{DRIVE0, EVENTS_TOTAL, start_metrics_server};
use std::net::SocketAddr;
use std::thread;

use crate::metrics::{DRIVE0_DELTA, DRIVE0_FRAME, DRIVE0_FRAME_DELTA, FRAMES_TOTAL};
use protozero_core::drive::{FrameCounter, drive_update};
use protozero_core::{DriveDiff, SensorEvent, run_sensor_loop};

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

    let _ = run_sensor_loop(|event: SensorEvent| {
        match event {
            SensorEvent::Data(chunk) => {
                for &b in chunk {
                    if b == b'\n' {
                        frame.on_boundary();
                        d_frame_prev = 0.0;
                        FRAMES_TOTAL.inc();
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
            }
            SensorEvent::Silence => {
                // Тишина: агент наблюдает отсутствие стимула.
                // Пока — только фиксируем факт. Decay будет в следующем ADR.
            }
        }
    });
}

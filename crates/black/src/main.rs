//! BLACK: немой, без логов/памяти.
//! ADR-016: non-blocking сенсор, агент «дышит» — различает стимул и тишину.
use protozero_core::drive::{FrameCounter, drive_update};
use protozero_core::{DriveDiff, SensorEvent, run_sensor_loop};

fn main() {
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
                        continue;
                    }

                    // глобальная динамика
                    d_global = drive_update(d_global);
                    let _dg = d_global_diff.step(d_global);

                    // покадровая динамика
                    let d_frame = frame.on_event();
                    let _df_delta = d_frame - d_frame_prev;
                    d_frame_prev = d_frame;

                    let _ = (&d_global, &_dg, &d_frame, &_df_delta);
                }
            }
            SensorEvent::Silence => {
                // Тишина: агент наблюдает отсутствие стимула.
                // Пока — только фиксируем факт. Decay будет в следующем ADR.
            }
        }
    });
}

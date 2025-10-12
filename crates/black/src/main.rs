//! BLACK: немой, без логов/памяти/времени.
//! Stage-2 (ADR-011): событийная регуляция drive_0 без параметров.
use protozero_core::{run_stdin_forever, DriveDiff};
use protozero_core::drive::{drive_update, FrameCounter};

fn main() {
    let mut d_global = 0.0;
    let mut d_global_diff = DriveDiff::new();

    let mut frame = FrameCounter::new();
    let mut d_frame_prev: f64 = 0.0;

    // Блокирующее чтение stdin; на каждый чанк увеличиваем drive_0 на +1.0.
    let _ = run_stdin_forever(|chunk: &[u8]| {
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
    });
}

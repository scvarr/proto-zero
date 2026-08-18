//! BLACK: сама сущность без observability.
//! ADR-027: оба исхода сравнения receptor причинно изменяют `drive_0`.

use memmap2::MmapOptions;
use std::{
    fs::File,
    hint::black_box,
    sync::atomic::{AtomicU8, Ordering},
};

const RECEPTOR_PATH: &str = "/receptor/cell";

fn main() {
    let file = File::open(RECEPTOR_PATH).expect("failed to open receptor cell");
    let receptor = MmapOptions::new()
        .len(1)
        .map_raw_read_only(&file)
        .expect("failed to map receptor cell");

    let receptor_cell = unsafe { AtomicU8::from_ptr(receptor.as_mut_ptr()) };
    let mut previous: Option<u8> = None;
    let mut drive_0: f64 = 0.0;

    loop {
        let current = receptor_cell.load(Ordering::Relaxed);

        if let Some(previous_value) = previous {
            let changed = current != previous_value;

            if changed {
                drive_0 += 1.0;
            } else {
                drive_0 -= 1.0;
            }

            // BLACK ничего не публикует. black_box не даёт оптимизатору
            // удалить немое внутреннее состояние drive_0.
            let _ = black_box(drive_0);
        }

        previous = Some(current);
    }
}

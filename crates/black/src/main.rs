//! BLACK: сама сущность без observability.
//! ADR-025: одна внешняя receptor cell и одна собственная память `previous`.

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

    loop {
        let current = receptor_cell.load(Ordering::Relaxed);

        if let Some(previous_value) = previous {
            let changed = current != previous_value;

            // BLACK ничего не публикует. black_box сохраняет само эфемерное вычисление
            // от удаления оптимизатором, не превращая changed в состояние агента.
            let _ = black_box(changed);
        }

        previous = Some(current);
    }
}

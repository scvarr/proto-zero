//! BLACK: сама сущность без observability.
//! ADR-025: одна внешняя receptor cell и одна собственная память `previous`.

use memmap2::MmapOptions;
use std::{fs::File, hint::black_box, ptr};

const RECEPTOR_PATH: &str = "/receptor/cell";

fn main() {
    let file = File::open(RECEPTOR_PATH).expect("failed to open receptor cell");
    let receptor = MmapOptions::new()
        .len(1)
        .map_raw_read_only(&file)
        .expect("failed to map receptor cell");

    let receptor_ptr = receptor.as_ptr();
    let mut previous: Option<u8> = None;

    loop {
        // Ячейка может быть изменена другим процессом в любой момент.
        // Volatile-read не позволяет компилятору заменить повторные чтения кешированным значением.
        let current = unsafe { ptr::read_volatile(receptor_ptr) };

        if let Some(previous_value) = previous {
            let changed = current != previous_value;

            // BLACK ничего не публикует. black_box сохраняет само эфемерное вычисление
            // от удаления оптимизатором, не превращая changed в состояние агента.
            let _ = black_box(changed);
        }

        previous = Some(current);
    }
}

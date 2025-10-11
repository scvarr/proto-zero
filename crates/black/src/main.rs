//! BLACK: немой, без логов/памяти/времени.
//! Stage-2 (ADR-011): событийная регуляция drive_0 без параметров.
use protozero_core::{run_stdin_forever};
use protozero_core::drive::drive_update;

fn main() {
    // Внутреннее состояние драйва (эфемерно: исчезает при завершении процесса).
    let mut drive_0: f64 = 0.0;

    // Блокирующее чтение stdin; на каждый чанк увеличиваем drive_0 на +1.0.
    let _ = run_stdin_forever(|_chunk| {
        // Симметрия: black так же накапливает, как и white, но без логов/выходов.
        drive_0 = drive_update(drive_0);
        let _ = &drive_0; // избегаем warning в релизе
    });
}

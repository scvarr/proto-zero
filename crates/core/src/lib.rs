pub mod drive;
pub use drive::{DriveDiff, drive_update};
use std::io::{self, Read};

/// Результат одного тика сенсора (ADR-016).
pub enum SensorEvent<'a> {
    /// Данные получены — стимул.
    Data(&'a [u8]),
    /// Попытка чтения не вернула данных — тишина.
    Silence,
}

/// Эфемерный сенсор: однократно «трогаем» stdin.
/// Возвращает true, если удалось прочитать хотя бы 1 байт (есть данные),
/// false — если EOF сразу, т.е. данных нет.
/// Оставляем для тестов/утилит
pub fn evaluate_stdin_once() -> io::Result<bool> {
    let mut stdin = io::stdin().lock();
    let mut buf = [0u8; 1];
    let n = stdin.read(&mut buf)?;
    Ok(n > 0)
}

/// «Жизнь» агента (ADR-006, устаревший API): блокирующее чтение.
/// Сохраняем для обратной совместимости.
pub fn run_stdin_forever<F>(mut on_chunk: F) -> io::Result<()>
where
    F: FnMut(&[u8]),
{
    let mut stdin = io::stdin().lock();
    let mut buf = vec![0u8; 1024];

    loop {
        let n = stdin.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }
        on_chunk(&buf[..n]);
    }
}

/// «Дыхание» агента (ADR-016): non-blocking тиковый цикл.
/// Каждая попытка чтения — один квант внутреннего времени.
/// on_tick получает Data при наличии данных, Silence при их отсутствии.
/// Возвращает Ok(()) при EOF (смерть).
#[cfg(unix)]
pub fn run_sensor_loop<F>(mut on_tick: F) -> io::Result<()>
where
    F: FnMut(SensorEvent),
{
    use std::os::unix::io::AsRawFd;

    let stdin = io::stdin();
    let fd = stdin.as_raw_fd();

    // Переводим stdin в non-blocking режим
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };

    let mut handle = stdin.lock();
    let mut buf = vec![0u8; 1024];

    loop {
        match handle.read(&mut buf) {
            Ok(0) => {
                // EOF — смерть.
                return Ok(());
            }
            Ok(n) => {
                on_tick(SensorEvent::Data(&buf[..n]));
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Тишина — данных нет, но pipe открыт.
                on_tick(SensorEvent::Silence);
            }
            Err(e) => return Err(e),
        }
    }
}

/// «Дыхание» агента (ADR-016): non-blocking тиковый цикл. Windows-версия.
#[cfg(windows)]
pub fn run_sensor_loop<F>(mut on_tick: F) -> io::Result<()>
where
    F: FnMut(SensorEvent),
{
    use std::io::BufRead;

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buf = vec![0u8; 1024];

    loop {
        // На Windows проверяем доступность данных через fill_buf
        match handle.fill_buf() {
            Ok([]) => {
                // EOF — смерть.
                return Ok(());
            }
            Ok(b) => {
                let n = b.len().min(buf.len());
                buf[..n].copy_from_slice(&b[..n]);
                handle.consume(n);
                on_tick(SensorEvent::Data(&buf[..n]));
                continue;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                on_tick(SensorEvent::Silence);
                continue;
            }
            Err(e) => return Err(e),
        };
    }
}

/// Обработка последовательности SensorEvent для тестирования агентной логики
/// без реального stdin (ADR-016).
pub fn process_events<F>(events: &[SensorEvent], mut on_tick: F)
where
    F: FnMut(SensorEvent),
{
    for event in events {
        match event {
            SensorEvent::Data(data) => on_tick(SensorEvent::Data(data)),
            SensorEvent::Silence => on_tick(SensorEvent::Silence),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use drive::drive_update;

    /// Инвариант: при потоке данных drive ведёт себя как раньше (монотонный рост).
    #[test]
    fn data_events_drive_monotone() {
        let mut d = 0.0;
        let data = vec![0x41u8; 100]; // 100 байт данных
        let events = [SensorEvent::Data(&data)];

        process_events(&events, |event| {
            if let SensorEvent::Data(chunk) = event {
                for &b in chunk {
                    if b != b'\n' {
                        let prev = d;
                        d = drive_update(d);
                        assert!(d > prev);
                        assert!(d >= 0.0 && d < 1.0);
                    }
                }
            }
        });

        assert!(d > 0.5);
    }

    /// Инвариант: Silence не меняет drive (пока нет decay).
    #[test]
    fn silence_does_not_change_drive() {
        let mut d = 0.0;
        // Сначала несколько событий
        for _ in 0..10 {
            d = drive_update(d);
        }
        let d_before_silence = d;

        // Тишина
        let events = [
            SensorEvent::Silence,
            SensorEvent::Silence,
            SensorEvent::Silence,
        ];
        process_events(&events, |event| {
            if let SensorEvent::Silence = event {
                // Пока decay не реализован — drive не меняется.
            }
        });

        assert_eq!(d, d_before_silence);
    }

    /// Инвариант: три состояния сенсора различимы.
    #[test]
    fn sensor_event_three_states() {
        let mut data_count = 0u64;
        let mut silence_count = 0u64;

        let data = [0x42u8; 5];
        let events = [
            SensorEvent::Data(&data),
            SensorEvent::Silence,
            SensorEvent::Silence,
            SensorEvent::Data(&data),
        ];

        process_events(&events, |event| match event {
            SensorEvent::Data(_) => data_count += 1,
            SensorEvent::Silence => silence_count += 1,
        });

        assert_eq!(data_count, 2);
        assert_eq!(silence_count, 2);
    }
}

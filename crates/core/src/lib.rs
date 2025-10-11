use std::io::{self, Read};

/// Эфемерный сенсор: однократно «трогаем» stdin.
/// Возвращает true, если удалось прочитать хотя бы 1 байт (есть данные),
/// false — если EOF сразу, т.е. данных нет.
pub fn evaluate_stdin_once() -> io::Result<bool> {
    let mut stdin = io::stdin().lock();
    let mut buf = [0u8; 1];
    let n = stdin.read(&mut buf)?;
    Ok(n > 0)
}

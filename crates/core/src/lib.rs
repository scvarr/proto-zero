pub mod drive;

use std::io::{self, Read};

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

/// «Жизнь» агента: бесконечная готовность на блокирующем чтении stdin.
/// Вызывает `on_chunk` на каждом успешно прочитанном ненулевом куске.
/// Возвращает Ok(()) при EOF (смерть) или ошибку ввода-вывода.
pub fn run_stdin_forever<F>(mut on_chunk: F) -> io::Result<()>
where
    F: FnMut(&[u8]),
{
    let mut stdin = io::stdin().lock();
    let mut buf = vec![0u8; 1024];

    loop {
        // Блокирующее чтение: нет времени, нет памяти.
        let n = stdin.read(&mut buf)?;
        if n == 0 {
            // EOF == завершение «жизни».
            return Ok(());
        }
        on_chunk(&buf[..n]);
    }
}


//! WORLD: отдельный процесс-источник байтов.
//! Stage-0: без маркировок данных. Односторонний поток в stdout.
//! Режимы:
//!   - stdin  : прокси stdin -> stdout (по умолчанию, удобно для тестов/ручной подачи)
//!   - urandom: бесконечный шум из /dev/urandom (остановка = SIGTERM/закрытие stdout)

use clap::Parser;
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Parser, Debug)]
#[command(name = "world", about = "Proto-Zero: world (Stage-0)", version)]
struct Args {
    /// Источник: "stdin" | "urandom"
    #[arg(long, default_value = "stdin")]
    source: String,

    /// Размер блока для urandom (байт). Игнорируется в режиме stdin.
    #[arg(long, default_value_t = 1024usize)]
    chunk: usize,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    match args.source.as_str() {
        "stdin" => pump_stdin_to_stdout(),
        "urandom" => pump_urandom_to_stdout(args.chunk),
        other => {
            eprintln!("unknown --source={other}, use stdin|urandom");
            std::process::exit(2);
        }
    }
}

/// Прозрачная прокачка stdin -> stdout (события контролируем руками/тестами).
fn pump_stdin_to_stdout() -> io::Result<()> {
    let mut inp = io::stdin().lock();
    let mut out = io::stdout().lock();
    let mut buf = [0u8; 8192];

    loop {
        let n = inp.read(&mut buf)?;
        if n == 0 {
            // EOF мира => завершение для подключённого агента (при пайпе).
            out.flush()?;
            return Ok(());
        }
        out.write_all(&buf[..n])?;
        out.flush()?; // даём байт(ы) немедленно
    }
}

/// Бесконечный шум из /dev/urandom (без таймеров).
fn pump_urandom_to_stdout(chunk: usize) -> io::Result<()> {
    let mut rng = File::open("/dev/urandom")?;
    let mut out = io::stdout().lock();
    let mut buf = vec![0u8; chunk.max(1)];

    loop {
        rng.read_exact(&mut buf)?;
        out.write_all(&buf)?;
        out.flush()?;
    }
}
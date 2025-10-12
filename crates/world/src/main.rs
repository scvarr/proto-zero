//! WORLD: отдельный процесс-источник байтов.
//! Stage-0: без маркировок данных. Односторонний поток в stdout.
//! Режимы:
//!   - stdin  : прокси stdin -> stdout (по умолчанию, удобно для тестов/ручной подачи)
//!   - urandom: бесконечный шум из /dev/urandom (остановка = SIGTERM/закрытие stdout)

use axum::{routing::get, Json, Router};
use bytes::BytesMut;
use parking_lot::RwLock;
use rand::{rngs::StdRng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{io::{self, Write, BufRead}, sync::Arc, time::Duration};
use tokio::{net::TcpListener, task};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    /// Бесконечный шум без маркеров.
    Continuous,
    /// Бурсты с семантической границей (вставка '\n').
    Bursty,
    /// Реплей файла (опц. с темпом); по строкам/чанкам.
    FileReplay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    mode: Mode,
    /// Байт/сек. Если 0 — без задержек (максимально быстро).
    bytes_per_sec: u64,
    /// Размер чанка записи (байт).
    chunk_size: usize,
    /// Размер кадра для Bursty (байт «полезных», после них — '\n').
    frame_bytes: usize,
    /// Джиттер задержки на чанк (мс), равномерный 0..=jitter_ms.
    jitter_ms: u64,
    /// Путь к файлу для FileReplay (читается построчно; '\n' сохраняется).
    file_path: String,
    /// Сид PRNG для детерминирования (0 — рандомный).
    seed: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Bursty,
            bytes_per_sec: 200_000, // 200 KB/s для удобного наблюдения
            chunk_size: 4096,
            frame_bytes: 32 * 1024, // 32 KB на кадр, затем '\n'
            jitter_ms: 50,
            file_path: String::new(),
            seed: 0,
        }
    }
}

#[derive(Clone)]
struct Shared {
    cfg: Arc<RwLock<Config>>,
}

// ---------- Точка входа ----------

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let shared = Shared { cfg: cfg.clone() };

    // HTTP API
    let api_shared = shared.clone();
    let app = Router::new()
        .route("/config", get(get_config).post(set_config))
        .with_state(api_shared);

    let http_task = task::spawn(async move {
        let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // Генератор в stdout (синхронный Writer приемлем: stdout буферизован Docker/FIFO)
    let gen_shared = shared.clone();
    let gen_task = task::spawn_blocking(move || run_generator(gen_shared));

    let _ = tokio::join!(http_task, gen_task);
    Ok(())
}

// ---------- HTTP обработчики ----------

async fn get_config(axum::extract::State(s): axum::extract::State<Shared>) -> Json<Config> {
    Json(s.cfg.read().clone())
}

async fn set_config(
    axum::extract::State(s): axum::extract::State<Shared>,
    Json(mut incoming): Json<Config>,
) -> Json<Config> {
    // Нормализация
    if incoming.chunk_size == 0 { incoming.chunk_size = 1024; }
    if incoming.frame_bytes == 0 { incoming.frame_bytes = 1024; }
    // Присваиваем без Result/poison, т.к. parking_lot
    *s.cfg.write() = incoming.clone();
    Json(incoming)
}

// ---------- Генератор ----------

fn run_generator(shared: Shared) -> anyhow::Result<()> {
    let mut stdout = io::BufWriter::new(io::stdout().lock());

    // Локальные рабочие переменные
    let seed = shared.cfg.read().seed;
    let mut rng = seeded_rng(seed);
    let mut bytes_in_frame: usize = 0;

    loop {
        let cfg = shared.cfg.read().clone();

        match cfg.mode {
            Mode::Continuous => {
                let n = cfg.chunk_size;
                write_random(&mut stdout, n, &mut rng)?;
                pacing(&cfg, n)?;
            }
            Mode::Bursty => {
                let n = cfg.chunk_size.min(cfg.frame_bytes.saturating_sub(bytes_in_frame));
                if n > 0 {
                    write_random(&mut stdout, n, &mut rng)?;
                    bytes_in_frame += n;
                    pacing(&cfg, n)?;
                }
                if bytes_in_frame >= cfg.frame_bytes {
                    stdout.write_all(b"\n")?;
                    stdout.flush()?;
                    bytes_in_frame = 0;
                    pacing(&cfg, 1)?;
                }
            }
            Mode::FileReplay => {
                if cfg.file_path.is_empty() {
                    let n = cfg.chunk_size;
                    write_random(&mut stdout, n, &mut rng)?;
                    pacing(&cfg, n)?;
                } else {
                    match std::fs::File::open(&cfg.file_path) {
                        Ok(mut f) => {
                            let mut reader = io::BufReader::new(&mut f);
                            let mut buf = Vec::with_capacity(8 * 1024);
                            loop {
                                buf.clear();
                                let n = reader.read_until(b'\n', &mut buf).unwrap_or(0);
                                if n == 0 { break; }
                                stdout.write_all(&buf)?;
                                stdout.flush()?;
                                pacing(&cfg, n)?;
                            }
                        }
                        Err(_) => {
                            let n = cfg.chunk_size;
                            write_random(&mut stdout, n, &mut rng)?;
                            pacing(&cfg, n)?;
                        }
                    }
                }
            }
        }
    }
}

/// Пишем n псевдослучайных байт через RngCore::fill_bytes (без метода gen).
fn write_random<W: Write>(mut w: W, n: usize, rng: &mut StdRng) -> io::Result<()> {
    let mut buf = BytesMut::zeroed(n);
    rng.fill_bytes(&mut buf[..]);
    w.write_all(&buf)?;
    Ok(())
}

/// Пейсинг по скорости и джиттеру. Если bytes_per_sec == 0 — без задержек.
fn pacing(cfg: &Config, n_bytes: usize) -> io::Result<()> {
    if cfg.bytes_per_sec == 0 { return Ok(()); }
    let base_ms = ((n_bytes as f64) / (cfg.bytes_per_sec as f64) * 1000.0) as u64;
    let jitter = if cfg.jitter_ms > 0 {
        // простой недетерминированный джиттер; при желании можно сделать детерминированным от seed
        rand::random::<u64>() % (cfg.jitter_ms + 1)
    } else { 0 };
    let total = base_ms.saturating_add(jitter);
    if total > 0 {
        std::thread::sleep(Duration::from_millis(total));
    }
    Ok(())
}

fn seeded_rng(seed: u64) -> StdRng {
    if seed == 0 {
        let mut seed_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut seed_bytes);
        StdRng::from_seed(seed_bytes)
    } else {
        // расширяем 8-байтовый сид до 32 байт
        let mut seed_bytes = [0u8; 32];
        seed_bytes[..8].copy_from_slice(&seed.to_le_bytes());
        StdRng::from_seed(seed_bytes)
    }
}
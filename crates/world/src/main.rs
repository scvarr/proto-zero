//! WORLD: лабораторная среда ADR-025.
//! Мир может только читать и изменять одну внешнюю receptor cell.

use axum::{Json, Router, extract::State, routing::get};
use memmap2::{MmapOptions, MmapRaw};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};
use tokio::net::TcpListener;

const RECEPTOR_PATH: &str = "/receptor/cell";

#[derive(Clone)]
struct Shared {
    receptor: Arc<MmapRaw>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct CellValue {
    value: u8,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(RECEPTOR_PATH)?;

    let receptor = MmapOptions::new().len(1).map_raw(&file)?;
    let shared = Shared {
        receptor: Arc::new(receptor),
    };

    let app = Router::new()
        .route("/cell", get(get_cell).put(put_cell))
        .with_state(shared);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_cell(State(state): State<Shared>) -> Json<CellValue> {
    Json(CellValue {
        value: receptor(&state).load(Ordering::Relaxed),
    })
}

async fn put_cell(
    State(state): State<Shared>,
    Json(incoming): Json<CellValue>,
) -> Json<CellValue> {
    receptor(&state).store(incoming.value, Ordering::Relaxed);
    Json(incoming)
}

fn receptor(state: &Shared) -> &AtomicU8 {
    unsafe { AtomicU8::from_ptr(state.receptor.as_mut_ptr()) }
}

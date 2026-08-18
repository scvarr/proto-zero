//! WHITE: анатомический двойник BLACK + внешняя лабораторная instrumentation.
//! ADR-025: анатомия остаётся `receptor -> compare with previous -> previous = current`.

mod metrics;

use memmap2::MmapOptions;
use std::{
    fs::File,
    net::SocketAddr,
    sync::atomic::{AtomicU8, Ordering},
    thread,
};

const RECEPTOR_PATH: &str = "/receptor/cell";

fn main() {
    let addr: SocketAddr = std::env::var("WHITE_METRICS_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9100".into())
        .parse()
        .expect("bad WHITE_METRICS_ADDR");

    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("failed to create metrics runtime");
        runtime.block_on(metrics::start_server(addr));
    });

    let file = File::open(RECEPTOR_PATH).expect("failed to open receptor cell");
    let receptor = MmapOptions::new()
        .len(1)
        .map_raw_read_only(&file)
        .expect("failed to map receptor cell");

    let receptor_cell = unsafe { AtomicU8::from_ptr(receptor.as_mut_ptr()) };
    let mut previous: Option<u8> = None;

    loop {
        let current = receptor_cell.load(Ordering::Relaxed);
        metrics::record_read(current);

        if let Some(previous_value) = previous {
            let changed = current != previous_value;
            metrics::record_comparison(previous_value, current, changed);
        }

        previous = Some(current);
        metrics::record_previous(current);
    }
}

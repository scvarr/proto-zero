use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    net::SocketAddr,
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::net::TcpListener;

const MAX_RECENT_CHANGES: usize = 512;

static HAS_CURRENT: AtomicBool = AtomicBool::new(false);
static CURRENT: AtomicU8 = AtomicU8::new(0);

static HAS_PREVIOUS: AtomicBool = AtomicBool::new(false);
static PREVIOUS: AtomicU8 = AtomicU8::new(0);

static HAS_COMPARISON: AtomicBool = AtomicBool::new(false);
static LAST_COMPARISON_PREVIOUS: AtomicU8 = AtomicU8::new(0);
static LAST_COMPARISON_CURRENT: AtomicU8 = AtomicU8::new(0);
static LAST_COMPARISON_CHANGED: AtomicBool = AtomicBool::new(false);

static READS_TOTAL: AtomicU64 = AtomicU64::new(0);
static CHANGES_TOTAL: AtomicU64 = AtomicU64::new(0);
static DRIVE_0_BITS: AtomicU64 = AtomicU64::new(0);

static RECENT_CHANGES: LazyLock<Mutex<VecDeque<ChangeRecord>>> =
    LazyLock::new(|| Mutex::new(VecDeque::with_capacity(MAX_RECENT_CHANGES)));

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChangeRecord {
    pub seq: u64,
    pub ts_ms: u64,
    pub previous: u8,
    pub current: u8,
    pub drive_0_after: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ComparisonSnapshot {
    pub previous: u8,
    pub current: u8,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabSnapshot {
    pub current: Option<u8>,
    pub previous: Option<u8>,
    pub last_comparison: Option<ComparisonSnapshot>,
    pub drive_0: f64,
    pub reads_total: u64,
    pub changes_total: u64,
    pub recent_changes: Vec<ChangeRecord>,
}

pub fn record_read(current: u8) {
    CURRENT.store(current, Ordering::Relaxed);
    HAS_CURRENT.store(true, Ordering::Release);
    READS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_comparison(previous: u8, current: u8, changed: bool, drive_0: f64) {
    DRIVE_0_BITS.store(drive_0.to_bits(), Ordering::Relaxed);
    LAST_COMPARISON_PREVIOUS.store(previous, Ordering::Relaxed);
    LAST_COMPARISON_CURRENT.store(current, Ordering::Relaxed);
    LAST_COMPARISON_CHANGED.store(changed, Ordering::Relaxed);
    HAS_COMPARISON.store(true, Ordering::Release);

    if !changed {
        return;
    }

    let seq = CHANGES_TOTAL.fetch_add(1, Ordering::Relaxed) + 1;
    let record = ChangeRecord {
        seq,
        ts_ms: now_ms(),
        previous,
        current,
        drive_0_after: drive_0,
    };

    let mut changes = RECENT_CHANGES.lock().expect("recent changes lock poisoned");
    if changes.len() == MAX_RECENT_CHANGES {
        changes.pop_front();
    }
    changes.push_back(record);
}

pub fn record_previous(previous: u8) {
    PREVIOUS.store(previous, Ordering::Relaxed);
    HAS_PREVIOUS.store(true, Ordering::Release);
}

pub async fn start_server(addr: SocketAddr) {
    let app = Router::new()
        .route("/state", get(state_handler))
        .route("/metrics", get(metrics_handler));

    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind WHITE instrumentation address");
    axum::serve(listener, app)
        .await
        .expect("WHITE instrumentation server crashed");
}

async fn state_handler() -> Json<LabSnapshot> {
    Json(snapshot())
}

async fn metrics_handler() -> String {
    let snapshot = snapshot();
    let current = snapshot.current.unwrap_or(0);
    let previous = snapshot.previous.unwrap_or(0);
    let previous_exists = u8::from(snapshot.previous.is_some());
    let last_changed = snapshot
        .last_comparison
        .map(|comparison| u8::from(comparison.changed))
        .unwrap_or(0);
    let comparison_exists = u8::from(snapshot.last_comparison.is_some());

    format!(
        concat!(
            "# TYPE protozero_receptor_current gauge\n",
            "protozero_receptor_current {current}\n",
            "# TYPE protozero_previous_value gauge\n",
            "protozero_previous_value {previous}\n",
            "# TYPE protozero_previous_exists gauge\n",
            "protozero_previous_exists {previous_exists}\n",
            "# TYPE protozero_last_changed gauge\n",
            "protozero_last_changed {last_changed}\n",
            "# TYPE protozero_comparison_exists gauge\n",
            "protozero_comparison_exists {comparison_exists}\n",
            "# TYPE protozero_drive_0 gauge\n",
            "protozero_drive_0 {drive_0}\n",
            "# TYPE protozero_reads_total counter\n",
            "protozero_reads_total {reads_total}\n",
            "# TYPE protozero_changes_total counter\n",
            "protozero_changes_total {changes_total}\n",
        ),
        current = current,
        previous = previous,
        previous_exists = previous_exists,
        last_changed = last_changed,
        comparison_exists = comparison_exists,
        drive_0 = snapshot.drive_0,
        reads_total = snapshot.reads_total,
        changes_total = snapshot.changes_total,
    )
}

fn snapshot() -> LabSnapshot {
    let current = HAS_CURRENT
        .load(Ordering::Acquire)
        .then(|| CURRENT.load(Ordering::Relaxed));
    let previous = HAS_PREVIOUS
        .load(Ordering::Acquire)
        .then(|| PREVIOUS.load(Ordering::Relaxed));
    let last_comparison = HAS_COMPARISON.load(Ordering::Acquire).then(|| ComparisonSnapshot {
        previous: LAST_COMPARISON_PREVIOUS.load(Ordering::Relaxed),
        current: LAST_COMPARISON_CURRENT.load(Ordering::Relaxed),
        changed: LAST_COMPARISON_CHANGED.load(Ordering::Relaxed),
    });

    let recent_changes = RECENT_CHANGES
        .lock()
        .expect("recent changes lock poisoned")
        .iter()
        .copied()
        .collect();

    LabSnapshot {
        current,
        previous,
        last_comparison,
        drive_0: f64::from_bits(DRIVE_0_BITS.load(Ordering::Relaxed)),
        reads_total: READS_TOTAL.load(Ordering::Relaxed),
        changes_total: CHANGES_TOTAL.load(Ordering::Relaxed),
        recent_changes,
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

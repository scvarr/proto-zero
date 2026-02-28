use axum::{
    Json, Router,
    response::{Html, IntoResponse, Sse, sse::Event},
    routing::get,
};
use futures_util::StreamExt;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{net::TcpListener, sync::broadcast, time::MissedTickBehavior};
use tokio_stream::wrappers::BroadcastStream;

const MAX_EVENTS: usize = 256;
const SILENCE_THRESHOLD_MS: u64 = 350;
const DEFAULT_BIND_ADDR: &str = "0.0.0.0:7070";
const DEFAULT_WORLD_CONFIG_URL: &str = "http://world:8080/config";
const DEFAULT_METRICS_URL: &str = "http://white:9100/metrics";
const INDEX_HTML: &str = include_str!("../static/index.html");

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct ObserverEvent {
    kind: EventKind,
    ts_ms: u64,
    size: Option<usize>,
    frame_boundaries: Option<usize>,
    duration_ms: Option<u64>,
    mode: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum EventKind {
    Chunk,
    SilenceGap,
    ConfigChanged,
    RelayStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
struct WorldConfig {
    mode: String,
    bytes_per_sec: u64,
    chunk_size: usize,
    frame_bytes: usize,
    jitter_ms: u64,
    file_path: String,
    seed: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
struct MetricsSnapshot {
    drive_0: Option<f64>,
    drive_delta: Option<f64>,
    events_total: Option<u64>,
    drive_0_frame: Option<f64>,
    drive_delta_frame: Option<f64>,
    frames_total: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct RelaySnapshot {
    bytes_total: u64,
    chunks_total: u64,
    frame_boundaries_total: u64,
    last_chunk_size: usize,
    last_gap_ms: Option<u64>,
    currently_silent: bool,
    relay_alive: bool,
}

impl Default for RelaySnapshot {
    fn default() -> Self {
        Self {
            bytes_total: 0,
            chunks_total: 0,
            frame_boundaries_total: 0,
            last_chunk_size: 0,
            last_gap_ms: None,
            currently_silent: false,
            relay_alive: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct StateResponse {
    now_ms: u64,
    relay: RelaySnapshot,
    metrics: MetricsSnapshot,
    world: Option<WorldConfig>,
    recent_events: Vec<ObserverEvent>,
}

#[derive(Debug)]
struct ObserverState {
    relay: RelaySnapshot,
    metrics: MetricsSnapshot,
    world: Option<WorldConfig>,
    recent_events: VecDeque<ObserverEvent>,
    last_chunk_at_ms: Option<u64>,
}

impl Default for ObserverState {
    fn default() -> Self {
        Self {
            relay: RelaySnapshot::default(),
            metrics: MetricsSnapshot::default(),
            world: None,
            recent_events: VecDeque::with_capacity(MAX_EVENTS),
            last_chunk_at_ms: None,
        }
    }
}

#[derive(Clone)]
struct AppState {
    inner: Arc<RwLock<ObserverState>>,
    tx: broadcast::Sender<ObserverEvent>,
}

impl AppState {
    fn push_event(&self, event: ObserverEvent) {
        {
            let mut state = self.inner.write();
            if state.recent_events.len() == MAX_EVENTS {
                state.recent_events.pop_front();
            }
            state.recent_events.push_back(event.clone());
        }
        let _ = self.tx.send(event);
    }

    fn snapshot(&self) -> StateResponse {
        let state = self.inner.read();
        StateResponse {
            now_ms: now_ms(),
            relay: state.relay.clone(),
            metrics: state.metrics.clone(),
            world: state.world.clone(),
            recent_events: state.recent_events.iter().cloned().collect(),
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let (tx, _) = broadcast::channel(512);
    let state = AppState {
        inner: Arc::new(RwLock::new(ObserverState::default())),
        tx,
    };

    let bind_addr: SocketAddr = std::env::var("OBSERVER_ADDR")
        .unwrap_or_else(|_| DEFAULT_BIND_ADDR.into())
        .parse()
        .expect("bad observer addr");
    let world_config_url = std::env::var("OBSERVER_WORLD_CONFIG_URL")
        .unwrap_or_else(|_| DEFAULT_WORLD_CONFIG_URL.into());
    let metrics_url =
        std::env::var("OBSERVER_METRICS_URL").unwrap_or_else(|_| DEFAULT_METRICS_URL.into());

    let relay_state = state.clone();
    let relay_handle = tokio::task::spawn_blocking(move || run_relay(relay_state));

    tokio::spawn(run_silence_monitor(state.clone()));
    tokio::spawn(run_world_poll(state.clone(), world_config_url));
    tokio::spawn(run_metrics_poll(state.clone(), metrics_url));

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/state", get(state_handler))
        .route("/api/events", get(events_handler))
        .route("/healthz", get(health_handler))
        .with_state(state.clone());

    let listener = TcpListener::bind(bind_addr).await?;
    let server = axum::serve(listener, app.into_make_service());

    tokio::select! {
        relay_result = relay_handle => {
            match relay_result {
                Ok(Ok(())) => Ok(()),
                Ok(Err(err)) => Err(err),
                Err(join_err) => Err(io::Error::other(format!("relay task crashed: {join_err}"))),
            }
        }
        server_result = server => {
            match server_result {
                Ok(()) => Ok(()),
                Err(err) => Err(io::Error::other(format!("observer server failed: {err}"))),
            }
        }
    }
}

async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn state_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<StateResponse> {
    Json(state.snapshot())
}

async fn health_handler() -> impl IntoResponse {
    "ok"
}

async fn events_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Sse<impl futures_util::stream::Stream<Item = Result<Event, axum::Error>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
        match msg {
            Ok(event) => {
                let json = serde_json::to_string(&event).ok()?;
                Some(Ok(Event::default().event("observer").data(json)))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream)
}

fn run_relay(state: AppState) -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    let mut buf = [0u8; 8192];

    loop {
        let n = stdin.read(&mut buf)?;
        if n == 0 {
            {
                let mut inner = state.inner.write();
                inner.relay.relay_alive = false;
                inner.relay.currently_silent = false;
            }
            state.push_event(ObserverEvent {
                kind: EventKind::RelayStatus,
                ts_ms: now_ms(),
                size: None,
                frame_boundaries: None,
                duration_ms: None,
                mode: None,
                message: Some("eof".into()),
            });
            return Ok(());
        }

        stdout.write_all(&buf[..n])?;
        stdout.flush()?;

        let ts = now_ms();
        let frame_boundaries = count_frame_boundaries(&buf[..n]);
        let mut gap = None;

        {
            let mut inner = state.inner.write();
            if let Some(prev_ts) = inner.last_chunk_at_ms {
                gap = ts.checked_sub(prev_ts);
            }
            inner.last_chunk_at_ms = Some(ts);
            inner.relay.bytes_total += n as u64;
            inner.relay.chunks_total += 1;
            inner.relay.frame_boundaries_total += frame_boundaries as u64;
            inner.relay.last_chunk_size = n;
            inner.relay.currently_silent = false;
            inner.relay.last_gap_ms = gap;
        }

        state.push_event(ObserverEvent {
            kind: EventKind::Chunk,
            ts_ms: ts,
            size: Some(n),
            frame_boundaries: Some(frame_boundaries),
            duration_ms: gap,
            mode: None,
            message: Some(hex_preview(&buf[..n])),
        });
    }
}

async fn run_silence_monitor(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        let maybe_gap = {
            let mut inner = state.inner.write();
            if !inner.relay.relay_alive {
                None
            } else if let Some(last_chunk_at_ms) = inner.last_chunk_at_ms {
                let gap = now_ms().saturating_sub(last_chunk_at_ms);
                if gap >= SILENCE_THRESHOLD_MS && !inner.relay.currently_silent {
                    inner.relay.currently_silent = true;
                    inner.relay.last_gap_ms = Some(gap);
                    Some(gap)
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(gap) = maybe_gap {
            state.push_event(ObserverEvent {
                kind: EventKind::SilenceGap,
                ts_ms: now_ms(),
                size: None,
                frame_boundaries: None,
                duration_ms: Some(gap),
                mode: None,
                message: Some("derived_external_gap".into()),
            });
        }
    }
}

async fn run_world_poll(state: AppState, url: String) {
    let client = reqwest::Client::new();
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        let response = client.get(&url).send().await;
        let Ok(response) = response else {
            continue;
        };
        let Ok(config) = response.json::<WorldConfig>().await else {
            continue;
        };

        let changed = {
            let mut inner = state.inner.write();
            let changed = inner.world.as_ref() != Some(&config);
            inner.world = Some(config.clone());
            changed
        };

        if changed {
            state.push_event(ObserverEvent {
                kind: EventKind::ConfigChanged,
                ts_ms: now_ms(),
                size: None,
                frame_boundaries: None,
                duration_ms: None,
                mode: Some(config.mode.clone()),
                message: Some("world_config_updated".into()),
            });
        }
    }
}

async fn run_metrics_poll(state: AppState, url: String) {
    let client = reqwest::Client::new();
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        let response = client.get(&url).send().await;
        let Ok(response) = response else {
            continue;
        };
        let Ok(body) = response.text().await else {
            continue;
        };
        let snapshot = parse_prometheus_metrics(&body);
        state.inner.write().metrics = snapshot;
    }
}

fn parse_prometheus_metrics(body: &str) -> MetricsSnapshot {
    let mut snapshot = MetricsSnapshot::default();

    for line in body.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(metric_name) = parts.next() else {
            continue;
        };
        let Some(metric_value) = parts.next() else {
            continue;
        };

        match metric_name {
            "protozero_drive_0" => snapshot.drive_0 = metric_value.parse().ok(),
            "protozero_drive_delta" => snapshot.drive_delta = metric_value.parse().ok(),
            "protozero_events_total" => snapshot.events_total = metric_value.parse().ok(),
            "protozero_drive_0_frame" => snapshot.drive_0_frame = metric_value.parse().ok(),
            "protozero_drive_delta_frame" => snapshot.drive_delta_frame = metric_value.parse().ok(),
            "protozero_frames_total" => snapshot.frames_total = metric_value.parse().ok(),
            _ => {}
        }
    }

    snapshot
}

fn count_frame_boundaries(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b == b'\n').count()
}

fn hex_preview(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_frame_boundaries_inside_chunk() {
        assert_eq!(count_frame_boundaries(b"abc\nxyz\n"), 2);
    }

    #[test]
    fn parses_prometheus_snapshot() {
        let body = "# HELP protozero_drive_0 x\nprotozero_drive_0 0.75\nprotozero_drive_delta 0.05\nprotozero_events_total 42\nprotozero_frames_total 3\n";
        let snapshot = parse_prometheus_metrics(body);

        assert_eq!(snapshot.drive_0, Some(0.75));
        assert_eq!(snapshot.drive_delta, Some(0.05));
        assert_eq!(snapshot.events_total, Some(42));
        assert_eq!(snapshot.frames_total, Some(3));
    }
}

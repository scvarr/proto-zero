use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Sse, sse::Event},
    routing::{get, put},
};
use futures_util::StreamExt;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{net::TcpListener, sync::broadcast, time::MissedTickBehavior};
use tokio_stream::wrappers::BroadcastStream;

const MAX_EVENTS: usize = 512;
const DEFAULT_BIND_ADDR: &str = "0.0.0.0:7070";
const DEFAULT_WORLD_CELL_URL: &str = "http://world:8080/cell";
const DEFAULT_WHITE_STATE_URL: &str = "http://white:9100/state";
const INDEX_HTML: &str = include_str!("../static/index.html");

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct CellValue {
    value: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct ChangeRecord {
    seq: u64,
    ts_ms: u64,
    previous: u8,
    current: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct ComparisonSnapshot {
    previous: u8,
    current: u8,
    changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WhiteSnapshot {
    current: Option<u8>,
    previous: Option<u8>,
    last_comparison: Option<ComparisonSnapshot>,
    reads_total: u64,
    changes_total: u64,
    recent_changes: Vec<ChangeRecord>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum EventKind {
    WorldWrite,
    WhiteChange,
}

#[derive(Debug, Clone, Serialize)]
struct ObserverEvent {
    kind: EventKind,
    ts_ms: u64,
    from: u8,
    to: u8,
    seq: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct StateResponse {
    now_ms: u64,
    world: Option<CellValue>,
    white: Option<WhiteSnapshot>,
    recent_events: Vec<ObserverEvent>,
}

#[derive(Debug, Default)]
struct ObserverState {
    world: Option<CellValue>,
    white: Option<WhiteSnapshot>,
    recent_events: VecDeque<ObserverEvent>,
    last_white_change_seq: u64,
}

#[derive(Clone)]
struct AppState {
    inner: Arc<RwLock<ObserverState>>,
    tx: broadcast::Sender<ObserverEvent>,
    client: reqwest::Client,
    world_cell_url: String,
    white_state_url: String,
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
            world: state.world,
            white: state.white.clone(),
            recent_events: state.recent_events.iter().cloned().collect(),
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let bind_addr: SocketAddr = std::env::var("OBSERVER_ADDR")
        .unwrap_or_else(|_| DEFAULT_BIND_ADDR.into())
        .parse()
        .expect("bad OBSERVER_ADDR");

    let world_cell_url = std::env::var("OBSERVER_WORLD_CELL_URL")
        .unwrap_or_else(|_| DEFAULT_WORLD_CELL_URL.into());
    let white_state_url = std::env::var("OBSERVER_WHITE_STATE_URL")
        .unwrap_or_else(|_| DEFAULT_WHITE_STATE_URL.into());

    let (tx, _) = broadcast::channel(1024);
    let state = AppState {
        inner: Arc::new(RwLock::new(ObserverState::default())),
        tx,
        client: reqwest::Client::new(),
        world_cell_url,
        white_state_url,
    };

    tokio::spawn(run_poll(state.clone()));

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/state", get(state_handler))
        .route("/api/events", get(events_handler))
        .route("/api/world/cell", put(world_cell_handler))
        .route("/healthz", get(health_handler))
        .with_state(state);

    let listener = TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await
}

async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn state_handler(State(state): State<AppState>) -> Json<StateResponse> {
    Json(state.snapshot())
}

async fn health_handler() -> impl IntoResponse {
    "ok"
}

async fn world_cell_handler(
    State(state): State<AppState>,
    Json(incoming): Json<CellValue>,
) -> Result<Json<CellValue>, (StatusCode, String)> {
    let before = fetch_world(&state).await.ok().flatten().unwrap_or(incoming);

    let response = state
        .client
        .put(&state.world_cell_url)
        .json(&incoming)
        .send()
        .await
        .map_err(bad_gateway)?;

    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("world returned {}", response.status()),
        ));
    }

    let applied = response.json::<CellValue>().await.map_err(bad_gateway)?;
    state.inner.write().world = Some(applied);
    state.push_event(ObserverEvent {
        kind: EventKind::WorldWrite,
        ts_ms: now_ms(),
        from: before.value,
        to: applied.value,
        seq: None,
    });

    Ok(Json(applied))
}

async fn events_handler(
    State(state): State<AppState>,
) -> Sse<impl futures_util::stream::Stream<Item = Result<Event, axum::Error>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|message| async move {
        match message {
            Ok(event) => {
                let json = serde_json::to_string(&event).ok()?;
                Some(Ok(Event::default().event("observer").data(json)))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream)
}

async fn run_poll(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        if let Ok(Some(world)) = fetch_world(&state).await {
            state.inner.write().world = Some(world);
        }

        let Ok(Some(white)) = fetch_white(&state).await else {
            continue;
        };

        let new_changes = {
            let mut inner = state.inner.write();
            let last_seq = inner.last_white_change_seq;
            let changes: Vec<_> = white
                .recent_changes
                .iter()
                .copied()
                .filter(|change| change.seq > last_seq)
                .collect();

            if let Some(last) = changes.last() {
                inner.last_white_change_seq = last.seq;
            }
            inner.white = Some(white);
            changes
        };

        for change in new_changes {
            state.push_event(ObserverEvent {
                kind: EventKind::WhiteChange,
                ts_ms: change.ts_ms,
                from: change.previous,
                to: change.current,
                seq: Some(change.seq),
            });
        }
    }
}

async fn fetch_world(state: &AppState) -> Result<Option<CellValue>, reqwest::Error> {
    let response = state.client.get(&state.world_cell_url).send().await?;
    if !response.status().is_success() {
        return Ok(None);
    }
    response.json::<CellValue>().await.map(Some)
}

async fn fetch_white(state: &AppState) -> Result<Option<WhiteSnapshot>, reqwest::Error> {
    let response = state.client.get(&state.white_state_url).send().await?;
    if !response.status().is_success() {
        return Ok(None);
    }
    response.json::<WhiteSnapshot>().await.map(Some)
}

fn bad_gateway(error: reqwest::Error) -> (StatusCode, String) {
    (StatusCode::BAD_GATEWAY, error.to_string())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

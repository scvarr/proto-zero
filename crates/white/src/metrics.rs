use axum::{routing::get, serve, Router};
use once_cell::sync::Lazy;
use prometheus::{Encoder, Gauge, IntCounter, Opts, Registry, TextEncoder};
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);
pub static DRIVE0: Lazy<Gauge> = Lazy::new(|| {
    let g = Gauge::with_opts(Opts::new("protozero_drive_0", "Drive-0 in [0,1)")).unwrap();
    REGISTRY.register(Box::new(g.clone())).unwrap();
    g
});
pub static EVENTS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let c = IntCounter::with_opts(Opts::new("protozero_events_total", "Total input events")).unwrap();
    REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buf = Vec::new();
    encoder.encode(&metric_families, &mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

pub async fn start_metrics_server(addr: SocketAddr) {
    let app = Router::new().route("/metrics", get(metrics_handler));
    let listener = TcpListener::bind(addr).await.expect("bind metrics addr");
    serve(listener, app).await.expect("metrics server crashed");
}

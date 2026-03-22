use prometheus::{Registry, Counter, opts, TextEncoder, Encoder};
use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref BIO_PULSE_COUNTER: Counter = Counter::with_opts(
        opts!("sentinel_media_bio_pulses_total", "Total de pulsos bio-sincrónicos enviados a Fenix")
    ).expect("No se pudo crear el contador de pulsos");
}

pub async fn start_metrics_server() {
    REGISTRY.register(Box::new(BIO_PULSE_COUNTER.clone())).ok();

    let app = Router::new().route("/metrics", get(metrics_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 9091));
    println!("📊 Servidor de Métricas de Media en http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

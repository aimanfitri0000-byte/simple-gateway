// src/main.rs
use axum::{
    Router, routing::get, routing::post, response::{Json, IntoResponse}, extract::State,
    http::StatusCode,
};
use std::net::SocketAddr;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use reqwest::Client;
use std::sync::Arc;
use prometheus::{Encoder, TextEncoder};
use tracing_subscriber::{fmt, EnvFilter};

mod auth;
mod middleware;
mod rate_limiter;
mod metrics;
mod logging;
mod service_registry;
mod consul_discovery;

use service_registry::{LoadBalancingStrategy, REGISTRY};
use consul_discovery::discover_services;

#[tokio::main]
async fn main() {
    // Inisialisasi logging & tracing
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let client = Arc::new(Client::new());

    let registry = REGISTRY.clone();

    // Dapatkan senarai awal dari Consul
    if let Err(e) = discover_services(registry.clone(), client.as_ref()).await {
        eprintln!("Consul discovery failed: {}", e);
    } else {
        tracing::info!("Initial service discovery from Consul succeeded");
    }

    // Task penyegaran setiap 30 saat
    let client_for_task = client.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = discover_services(registry.clone(), client_for_task.as_ref()).await {
                eprintln!("Consul refresh failed: {}", e);
            }
        }
    });

    let protected_routes = Router::new()
        .route("/api/users", get(get_users))
        .layer(axum::middleware::from_fn(middleware::auth_middleware));

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/login", post(login))
        .route("/metrics", get(metrics_handler))
        .nest("/", protected_routes)
        .layer(axum::middleware::from_fn(logging::logging_middleware))
        .layer(axum::middleware::from_fn(rate_limiter::rate_limit_middleware))
        .with_state(client);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 API Gateway running on http://{}", addr);
    println!("📝 Try: http://localhost:3000/api/users (perlu token)");
    println!("🔑 Login: POST http://localhost:3000/login dengan JSON: {{\"username\":\"alice\",\"password\":\"password123\"}}");
    println!("📊 Metrics: http://localhost:3000/metrics");

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Json<Value> {
    Json(json!({
        "message": "API Gateway with Authentication & Service Discovery",
        "status": "running"
    }))
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "healthy" }))
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    match auth::authenticate(&payload.username, &payload.password) {
        Some(token) => Json(json!({ "token": token })).into_response(),
        None => (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    }
}

async fn get_users(
    State(client): State<Arc<Client>>,
) -> Result<Json<Value>, StatusCode> {
    let mut registry = REGISTRY.lock().await;
    let instance = registry.select_instance("users", LoadBalancingStrategy::RoundRobin);
    drop(registry);

    let instance = match instance {
        Some(i) => i,
        None => return Err(StatusCode::SERVICE_UNAVAILABLE),
    };

    let url = format!("{}/users", instance.url);
    match client.get(&url).send().await {
        Ok(resp) => {
            let data = resp.json::<Value>().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(data))
        }
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
#![allow(dead_code)]

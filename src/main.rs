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
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::{Utc, Duration};

mod auth;
mod middleware;
mod rate_limiter;
mod metrics;
mod logging;
mod service_registry;
mod consul_discovery;
mod message_queue;

use service_registry::{LoadBalancingStrategy, REGISTRY};
use consul_discovery::discover_services;
use message_queue::{MessageQueue, start_worker};

// Manual cache
struct Cache {
    data: Mutex<HashMap<String, (Value, i64)>>,
}

impl Cache {
    fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }
    
    fn get(&self, key: &str) -> Option<Value> {
        let cache = self.data.lock().unwrap();
        if let Some((value, expires_at)) = cache.get(key) {
            if Utc::now().timestamp() < *expires_at {
                return Some(value.clone());
            }
        }
        None
    }
    
    fn set(&self, key: &str, value: Value, ttl_secs: u64) {
        let expires_at = Utc::now().timestamp() + ttl_secs as i64;
        let mut cache = self.data.lock().unwrap();
        cache.insert(key.to_string(), (value, expires_at));
    }
}

#[tokio::main]
async fn main() {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let client = Arc::new(Client::new());
    let cache = Arc::new(Cache::new());
    let mq = MessageQueue::new();
    start_worker(mq.clone());

    let registry = REGISTRY.clone();

    if let Err(e) = discover_services(registry.clone(), client.as_ref()).await {
        eprintln!("Consul discovery failed: {}", e);
    } else {
        tracing::info!("Initial service discovery from Consul succeeded");
    }

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
        .route("/api/users", get(get_users_handler))
        .layer(axum::middleware::from_fn(middleware::auth_middleware));

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/login", post(login_handler))
        .route("/metrics", get(metrics_handler))
        .nest("/", protected_routes)
        .layer(axum::middleware::from_fn(logging::logging_middleware))
        .layer(axum::middleware::from_fn(rate_limiter::rate_limit_middleware))
        .with_state((client, cache, mq));

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
        "message": "API Gateway with Cache & Message Queue",
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

async fn login_handler(
    State((_, _, mq)): State<(Arc<Client>, Arc<Cache>, MessageQueue)>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match auth::authenticate(&payload.username, &payload.password) {
        Some(token) => {
            mq.publish("user_logged_in", json!({
                "username": payload.username,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })).await;
            Json(json!({ "token": token })).into_response()
        }
        None => (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    }
}

async fn get_users_handler(
    State((client, cache, mq)): State<(Arc<Client>, Arc<Cache>, MessageQueue)>,
) -> Result<Json<Value>, StatusCode> {
    mq.publish("api_called", json!({
        "endpoint": "/api/users",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })).await;

    // Check cache
    let cache_key = "users:all";
    if let Some(cached_data) = cache.get(cache_key) {
        println!("🟢 Cache HIT - returning cached data");
        return Ok(Json(cached_data));
    }

    println!("🟡 Cache MISS - calling microservice...");
    match client.get("http://localhost:8001/users").send().await {
        Ok(resp) => {
            let data = resp.json::<Value>().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            cache.set(cache_key, data.clone(), 60);
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
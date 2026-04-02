use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub gateway: GatewayConfig,
    pub services: HashMap<String, ServiceConfig>,
    pub auth: AuthConfig,
    pub rate_limits: HashMap<String, RateLimitConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GatewayConfig {
    pub port: u16,
    pub timeout_seconds: u64,
    pub max_body_size_mb: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub discovery_type: DiscoveryType,
    pub endpoints: Vec<Endpoint>,
    pub load_balancing: LoadBalancingStrategy,
    pub circuit_breaker: CircuitBreakerConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Endpoint {
    pub path: String,
    pub methods: Vec<String>,
    pub upstream_path: String,
    pub rate_limit_key: Option<String>,
    pub auth_required: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DiscoveryType {
    Static,
    Consul,
    Kubernetes,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    Random,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout_seconds: u64,
    pub half_open_requests: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&content)?)
    }
}
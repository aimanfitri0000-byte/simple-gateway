// src/metrics/mod.rs
use prometheus::{Registry, Counter, Histogram, Encoder, TextEncoder};
use lazy_static::lazy_static;

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
    static ref REQUEST_COUNTER: Counter = Counter::new("requests_total", "Total requests").unwrap();
    static ref REQUEST_DURATION: Histogram = Histogram::new("request_duration_seconds", "Request duration").unwrap();
}

pub struct MetricsRegistry;

impl MetricsRegistry {
    pub fn new() -> Self {
        REGISTRY.register(Box::new(REQUEST_COUNTER.clone())).unwrap();
        REGISTRY.register(Box::new(REQUEST_DURATION.clone())).unwrap();
        Self
    }

    pub fn register(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn record_request(&self, service: &str, method: &str, status: u16, duration: f64) {
        REQUEST_COUNTER.inc();
        REQUEST_DURATION.observe(duration);
    }

    pub fn record_error(&self, service: &str, method: &str) {
        // implement if needed
    }

    pub fn record_auth_failure(&self, service: &str) {
        // implement if needed
    }
}
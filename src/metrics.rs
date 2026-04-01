// src/metrics.rs
use prometheus::{register_counter, register_histogram_vec, Counter, HistogramVec};
use lazy_static::lazy_static;

lazy_static! {
    // Jumlah request mengikut method, endpoint, status
    pub static ref REQUESTS: Counter = register_counter!(
        "gateway_requests_total",
        "Total number of HTTP requests"
    ).unwrap();

    // Tempoh request (histogram)
    pub static ref REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "gateway_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "endpoint", "status"]
    ).unwrap();
}


// src/logging.rs
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{error, info, span, warn, Level};

pub async fn logging_middleware(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let start = Instant::now();

    // Buat span untuk tracing (akan muncul dalam log)
    let span = span!(Level::INFO, "request", method = %method, path = %path);
    let _enter = span.enter();

    // Teruskan request ke middleware seterusnya
    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();

    // Log mengikut status
    match status {
        StatusCode::TOO_MANY_REQUESTS => warn!(
            "{} {} - {} - {:?} - rate limited",
            method, path, status, duration
        ),
        s if s.is_client_error() => warn!("{} {} - {} - {:?}", method, path, status, duration),
        s if s.is_server_error() => error!("{} {} - {} - {:?}", method, path, status, duration),
        _ => info!("{} {} - {} - {:?}", method, path, status, duration),
    }

    // Rekod metrics
    use crate::metrics::{REQUESTS, REQUEST_DURATION};
    REQUESTS.inc();
    REQUEST_DURATION
        .with_label_values(&[method.as_str(), &path, status.as_str()])
        .observe(duration.as_secs_f64());

    response
}

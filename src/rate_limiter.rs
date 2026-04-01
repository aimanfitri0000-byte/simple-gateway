// src/rate_limiter.rs
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use governor::{
    clock::QuantaClock, middleware::NoOpMiddleware, state::keyed::DashMapStateStore, Quota,
    RateLimiter,
};
use lazy_static::lazy_static;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

// Define type supaya tak serabut
type Limiter = RateLimiter<
    IpAddr,
    DashMapStateStore<IpAddr>,
    QuantaClock,
    NoOpMiddleware<governor::clock::QuantaInstant>,
>;

// Rate limiter global: setiap IP dibenarkan 10 request per minit
lazy_static! {
    static ref RATE_LIMITER: Arc<Limiter> = {
        let quota = Quota::per_minute(NonZeroU32::new(10).unwrap());
        Arc::new(RateLimiter::keyed(quota))
    };
}

pub async fn rate_limit_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // Ambil IP
    let ip = req
        .extensions()
        .get::<IpAddr>()
        .copied()
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|ip_str| ip_str.parse::<IpAddr>().ok())
        })
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap());

    // Check rate limit
    match RATE_LIMITER.check_key(&ip) {
        Ok(_) => Ok(next.run(req).await),
        Err(_) => Err(StatusCode::TOO_MANY_REQUESTS),
    }
}

// src/rate_limiter.rs
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use governor::state::keyed::DashMapStateStore;
use governor::{clock::DefaultClock, Quota, RateLimiter};
use lazy_static::lazy_static;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

// Rate limiter menggunakan DashMap untuk menyimpan state setiap IP
lazy_static! {
    static ref RATE_LIMITER: Arc<RateLimiter<IpAddr, DashMapStateStore<IpAddr>, DefaultClock>> = {
        let quota = Quota::per_minute(NonZeroU32::new(10).unwrap());
        Arc::new(RateLimiter::keyed(quota))
    };
}

pub async fn rate_limit_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // Dapatkan IP dari request
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

    // Semak rate limit
    match RATE_LIMITER.check_key(&ip) {
        Ok(_) => Ok(next.run(req).await),
        Err(_) => Err(StatusCode::TOO_MANY_REQUESTS),
    }
}

// ========== UNIT TESTS ==========
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn test_rate_limiter_allows_requests_within_limit() {
        let ip = IpAddr::from_str("127.0.0.1").unwrap();

        for _ in 0..10 {
            let result = RATE_LIMITER.check_key(&ip);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_blocks_after_limit() {
        let ip = IpAddr::from_str("192.168.1.1").unwrap();

        for _ in 0..10 {
            let _ = RATE_LIMITER.check_key(&ip);
        }

        let result = RATE_LIMITER.check_key(&ip);
        assert!(result.is_err());
    }

    #[test]
    fn test_rate_limiter_different_ips_separate() {
        let ip1 = IpAddr::from_str("10.0.0.1").unwrap();
        let ip2 = IpAddr::from_str("10.0.0.2").unwrap();

        for _ in 0..10 {
            let _ = RATE_LIMITER.check_key(&ip1);
        }

        let result = RATE_LIMITER.check_key(&ip2);
        assert!(result.is_ok());

        let result = RATE_LIMITER.check_key(&ip1);
        assert!(result.is_err());
    }
}

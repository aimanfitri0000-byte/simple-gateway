// src/gateway/middleware.rs
use axum::{
    middleware::Next,
    response::Response,
    http::Request,
};

pub struct AuthMiddleware;

impl AuthMiddleware {
    pub async fn auth<B>(req: Request<B>, next: Next<B>) -> Response {
        // Sementara, teruskan request tanpa auth
        next.run(req).await
    }
}
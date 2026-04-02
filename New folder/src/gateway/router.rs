use axum::{
    Router,
    routing::{get, post, put, delete},
    middleware,
};
use std::sync::Arc;
use tower_http::{
    trace::TraceLayer,
    cors::{CorsLayer, Any},
    timeout::TimeoutLayer,
    limit::RequestBodyLimitLayer,
};

use crate::{config::Config, services::discovery::ServiceRegistry, metrics::MetricsRegistry};
use super::{handler::GatewayHandler, middleware::AuthMiddleware};

pub async fn create_router(
    config: Config,
    service_registry: ServiceRegistry,
    metrics: MetricsRegistry,
) -> anyhow::Result<Router> {
    let handler = Arc::new(GatewayHandler::new(
        config.clone(),
        service_registry,
        metrics,
    ));

    let mut router = Router::new();

    // Dynamically register routes based on configuration
    for (service_name, service_config) in &config.services {
        for endpoint in &service_config.endpoints {
            let handler_clone = handler.clone();
            let service_name_clone = service_name.clone();
            let endpoint_clone = endpoint.clone();

            let route_handler = move |req| {
                let handler = handler_clone.clone();
                let service = service_name_clone.clone();
                let endpoint = endpoint_clone.clone();
                async move {
                    handler.handle(req, service, endpoint).await
                }
            };

            // Add routes for each HTTP method
            for method in &endpoint.methods {
                router = match method.as_str() {
                    "GET" => router.route(&endpoint.path, get(route_handler.clone())),
                    "POST" => router.route(&endpoint.path, post(route_handler.clone())),
                    "PUT" => router.route(&endpoint.path, put(route_handler.clone())),
                    "DELETE" => router.route(&endpoint.path, delete(route_handler.clone())),
                    _ => router,
                };
            }
        }
    }

    // Apply global middleware
    let app = router
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive()
            .allow_methods(Any)
            .allow_headers(Any))
        .layer(TimeoutLayer::new(
            std::time::Duration::from_secs(config.gateway.timeout_seconds)
        ))
        .layer(RequestBodyLimitLayer::new(
            config.gateway.max_body_size_mb * 1024 * 1024
        ))
        .layer(middleware::from_fn(AuthMiddleware::auth));

    Ok(app)
}

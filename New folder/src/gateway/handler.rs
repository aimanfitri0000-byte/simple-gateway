use axum::{
    extract::{Request, State},
    response::{Response, IntoResponse},
    body::Body,
    http::{StatusCode, HeaderMap},
};
use std::sync::Arc;
use tracing::{info, error, span, Instrument};
use reqwest::Client;
use chrono::Utc;

use crate::{
    config::{Config, Endpoint},
    services::discovery::ServiceRegistry,
    metrics::MetricsRegistry,
    auth::jwt::validate_token,
};

pub struct GatewayHandler {
    config: Config,
    service_registry: Arc<ServiceRegistry>,
    http_client: Client,
    metrics: Arc<MetricsRegistry>,
}

impl GatewayHandler {
    pub fn new(
        config: Config,
        service_registry: ServiceRegistry,
        metrics: MetricsRegistry,
    ) -> Self {
        Self {
            config,
            service_registry: Arc::new(service_registry),
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            metrics: Arc::new(metrics),
        }
    }

    pub async fn handle(
        self: Arc<Self>,
        mut req: Request,
        service_name: String,
        endpoint: Endpoint,
    ) -> Response {
        let span = span!(tracing::Level::INFO, "gateway_request", 
            service = %service_name,
            path = %req.uri().path(),
            method = %req.method(),
        );
        
        async move {
            // Record start time
            let start_time = Utc::now();
            
            // Authentication
            if endpoint.auth_required {
                if let Err(e) = validate_token(&req, &self.config.auth.jwt_secret) {
                    self.metrics.record_auth_failure(&service_name);
                    return (StatusCode::UNAUTHORIZED, format!("Authentication failed: {}", e)).into_response();
                }
            }

            // Rate limiting
            if let Some(rate_limit_key) = &endpoint.rate_limit_key {
                // Implementation would use governor for rate limiting
                // Simplified for brevity
            }

            // Service discovery - get available instances
            let instances = match self.service_registry.get_instances(&service_name).await {
                Ok(instances) => instances,
                Err(e) => {
                    error!("Service discovery failed: {}", e);
                    return (StatusCode::SERVICE_UNAVAILABLE, "Service unavailable").into_response();
                }
            };

            if instances.is_empty() {
                error!("No available instances for service: {}", service_name);
                return (StatusCode::SERVICE_UNAVAILABLE, "No healthy instances").into_response();
            }

            // Load balancing - select instance
            let instance = match self.service_registry.select_instance(&service_name, &instances).await {
                Some(instance) => instance,
                None => {
                    return (StatusCode::SERVICE_UNAVAILABLE, "No instance selected").into_response();
                }
            };

            // Build upstream URL
            let upstream_url = format!(
                "{}{}",
                instance.url.trim_end_matches('/'),
                endpoint.upstream_path
            );

            // Forward request
            let mut upstream_req = self.http_client
                .request(req.method().clone(), &upstream_url)
                .headers(req.headers().clone())
                .body(req.into_body());

            // Add tracing headers
            upstream_req = upstream_req.header("X-Request-ID", uuid::Uuid::new_v4().to_string());

            // Execute request
            match upstream_req.send().await {
                Ok(response) => {
                    // Record metrics
                    let duration = (Utc::now() - start_time).num_milliseconds();
                    self.metrics.record_request(
                        &service_name,
                        req.method().as_str(),
                        response.status().as_u16(),
                        duration as f64,
                    );

                    // Transform response if needed
                    let mut response_builder = Response::builder()
                        .status(response.status())
                        .headers(response.headers().clone());
                    
                    // Add gateway headers
                    response_builder = response_builder
                        .header("X-Gateway", "rust-gateway")
                        .header("X-Upstream", &instance.id);

                    match response_builder.body(Body::from_stream(response.bytes_stream())) {
                        Ok(resp) => resp.into_response(),
                        Err(e) => {
                            error!("Failed to build response: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Gateway error").into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("Upstream request failed: {}", e);
                    self.metrics.record_error(&service_name, req.method().as_str());
                    (StatusCode::BAD_GATEWAY, format!("Upstream error: {}", e)).into_response()
                }
            }
        }
        .instrument(span)
        .await
    }
}
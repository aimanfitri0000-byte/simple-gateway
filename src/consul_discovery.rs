// src/consul_discovery.rs
use crate::service_registry::{ServiceInstance, ServiceRegistry};
use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub async fn discover_services(
    registry: Arc<Mutex<ServiceRegistry>>,
    client: &Client,
) -> Result<(), Box<dyn Error>> {
    let url = "http://localhost:8500/v1/health/service/users";
    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        return Err(format!("Consul API returned {}", resp.status()).into());
    }

    let entries: Vec<Value> = resp.json().await?;
    let mut new_instances = Vec::new();

    for entry in entries {
        // Gunakan &[] sebagai fallback (static)
        let checks = entry["Checks"].as_array().map(Vec::as_slice).unwrap_or(&[]);
        let healthy = checks.iter().all(|c| c["Status"] == "passing");
        if !healthy {
            continue;
        }

        let service = &entry["Service"];
        let id = service["ID"].as_str().unwrap_or("").to_string();
        let address = service["Address"].as_str().unwrap_or("127.0.0.1");
        let port = service["Port"].as_u64().unwrap_or(8001);

        new_instances.push(ServiceInstance {
            id,
            url: format!("http://{}:{}", address, port),
            healthy: true,
            connections: 0,
        });
    }

    if !new_instances.is_empty() {
        let mut reg = registry.lock().await;
        reg.register("users", new_instances);
        info!("Updated users service instances from Consul");
    }

    Ok(())
}

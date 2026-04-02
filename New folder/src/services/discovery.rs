use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::config::{Config, LoadBalancingStrategy};
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct ServiceInstance {
    pub id: String,
    pub url: String,
    pub healthy: bool,
    pub connections: usize,
}

pub struct ServiceRegistry {
    instances: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
    config: Config,
}

impl ServiceRegistry {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let registry = Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
        };
        
        registry.discover_services().await?;
        registry.start_health_check();
        
        Ok(registry)
    }
    
    async fn discover_services(&self) -> anyhow::Result<()> {
        let mut instances = self.instances.write().await;
        
        for (service_name, service_config) in &self.config.services {
            match service_config.discovery_type {
                crate::config::DiscoveryType::Static => {
                    // Static service discovery from config
                    let static_instances = vec![
                        ServiceInstance {
                            id: format!("{}-1", service_name),
                            url: format!("http://localhost:8080/{}", service_name),
                            healthy: true,
                            connections: 0,
                        }
                    ];
                    instances.insert(service_name.clone(), static_instances);
                }
                crate::config::DiscoveryType::Consul => {
                    // Consul integration would go here
                    info!("Consul discovery for {}", service_name);
                }
                crate::config::DiscoveryType::Kubernetes => {
                    // Kubernetes API integration
                    info!("Kubernetes discovery for {}", service_name);
                }
            }
        }
        
        Ok(())
    }
    
    fn start_health_check(&self) {
        let instances = self.instances.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                // Implement health checks for instances
                info!("Running health checks...");
            }
        });
    }
    
    pub async fn get_instances(&self, service_name: &str) -> anyhow::Result<Vec<ServiceInstance>> {
        let instances = self.instances.read().await;
        instances.get(service_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Service not found: {}", service_name))
    }
    
    pub async fn select_instance(&self, service_name: &str, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        let service_config = self.config.services.get(service_name)?;
        
        let healthy_instances: Vec<ServiceInstance> = instances
            .iter()
            .filter(|i| i.healthy)
            .cloned()
            .collect();
        
        if healthy_instances.is_empty() {
            return None;
        }
        
        match service_config.load_balancing {
            LoadBalancingStrategy::RoundRobin => {
                // Simple round-robin implementation
                static mut COUNTER: usize = 0;
                unsafe {
                    let idx = COUNTER % healthy_instances.len();
                    COUNTER += 1;
                    Some(healthy_instances[idx].clone())
                }
            }
            LoadBalancingStrategy::LeastConnections => {
                healthy_instances
                    .iter()
                    .min_by_key(|i| i.connections)
                    .cloned()
            }
            LoadBalancingStrategy::Random => {
                use rand::seq::SliceRandom;
                healthy_instances.choose(&mut rand::thread_rng()).cloned()
            }
        }
    }
}
// src/service_registry.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use lazy_static::lazy_static;
use rand::seq::SliceRandom; // untuk Random strategy

#[derive(Debug, Clone)]
pub struct ServiceInstance {
    pub id: String,
    pub url: String,
    pub healthy: bool,
    pub connections: usize, // untuk Least Connections (optional)
}

pub enum LoadBalancingStrategy {
    RoundRobin,
    Random,
    LeastConnections,
}

pub struct ServiceRegistry {
    instances: HashMap<String, Vec<ServiceInstance>>,
    current_index: HashMap<String, usize>, // untuk round-robin
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            current_index: HashMap::new(),
        }
    }

    // Daftar satu atau lebih instance untuk satu service
    pub fn register(&mut self, service_name: &str, instances: Vec<ServiceInstance>) {
        self.instances.insert(service_name.to_string(), instances);
        self.current_index.insert(service_name.to_string(), 0);
    }

    // Pilih instance berdasarkan strategi
    pub fn select_instance(
        &mut self,
        service_name: &str,
        strategy: LoadBalancingStrategy,
    ) -> Option<ServiceInstance> {
        let instances = self.instances.get(service_name)?;
        if instances.is_empty() {
            return None;
        }

        // Filter yang sihat
        let healthy: Vec<&ServiceInstance> = instances.iter().filter(|i| i.healthy).collect();
        if healthy.is_empty() {
            return None;
        }

        match strategy {
            LoadBalancingStrategy::RoundRobin => {
                let idx = self.current_index.get(service_name).unwrap_or(&0);
                let selected = healthy[*idx % healthy.len()].clone();
                let new_idx = (*idx + 1) % healthy.len();
                self.current_index.insert(service_name.to_string(), new_idx);
                Some(selected)
            }
            LoadBalancingStrategy::Random => {
                let mut rng = rand::thread_rng();
                healthy.choose(&mut rng).map(|&i| i.clone())
            }
            LoadBalancingStrategy::LeastConnections => {
                healthy
                    .iter()
                    .min_by_key(|i| i.connections)
                    .map(|&i| i.clone())
            }
        }
    }

    // Tandakan instance sebagai tidak sihat (optional untuk health check)
    pub fn mark_unhealthy(&mut self, service_name: &str, instance_id: &str) {
        if let Some(instances) = self.instances.get_mut(service_name) {
            for i in instances {
                if i.id == instance_id {
                    i.healthy = false;
                }
            }
        }
    }
}

lazy_static! {
    pub static ref REGISTRY: Arc<Mutex<ServiceRegistry>> = Arc::new(Mutex::new(ServiceRegistry::new()));
}
// src/message_queue.rs
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct MessageQueue {
    queue: Arc<Mutex<VecDeque<Value>>>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    // Hantar message ke queue
    pub async fn publish(&self, event: &str, data: Value) {
        let message = json!({
            "event": event,
            "data": data,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        let mut queue = self.queue.lock().await;
        queue.push_back(message);
        println!("📤 Published event: {}", event);
    }

    // Ambil message dari queue (FIFO)
    pub async fn consume(&self) -> Option<Value> {
        let mut queue = self.queue.lock().await;
        queue.pop_front()
    }
}

// Background worker untuk process queue
pub fn start_worker(queue: MessageQueue) {
    tokio::spawn(async move {
        loop {
            if let Some(message) = queue.consume().await {
                println!("📥 Processing: {}", serde_json::to_string_pretty(&message).unwrap());
                
                // Process based on event type
                if let Some(event) = message["event"].as_str() {
                    match event {
                        "user_logged_in" => {
                            let user = &message["data"];
                            println!("   → Send welcome email to: {}", user["username"]);
                            println!("   → Update last login timestamp");
                            println!("   → Send notification to admin");
                        }
                        "api_called" => {
                            let endpoint = &message["data"]["endpoint"];
                            println!("   → Log API call to analytics: {}", endpoint);
                        }
                        _ => println!("   → Unknown event: {}", event),
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });
}
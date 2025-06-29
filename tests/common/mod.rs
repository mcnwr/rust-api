use serde_json::Value;
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

pub mod mock_rabbitmq;
pub mod test_helpers;

/// Test configuration constants
pub const TEST_TIMEOUT: Duration = Duration::from_secs(30);
pub const MOCK_RABBITMQ_URL: &str = "amqp://guest:guest@localhost:5672/%2f";
pub const TEST_QUEUE_NAME: &str = "test_queue";

/// Common test data structures
#[derive(Debug, Clone)]
pub struct TestMessage {
    pub producer_id: u32,
    pub task_id: u32,
    pub message: String,
}

impl TestMessage {
    pub fn new(producer_id: u32, task_id: u32, message: String) -> Self {
        Self {
            producer_id,
            task_id,
            message,
        }
    }

    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "producer_id": self.producer_id,
            "task_id": self.task_id,
            "message": self.message
        })
    }
}

/// Test environment setup helpers
pub struct TestEnvironment {
    pub temp_dir: tempfile::TempDir,
    pub env_vars: HashMap<String, String>,
}

impl TestEnvironment {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let mut env_vars = HashMap::new();

        // Set test environment variables
        env_vars.insert("RABBITMQ_URL".to_string(), MOCK_RABBITMQ_URL.to_string());
        env_vars.insert("QUEUE_NAME".to_string(), TEST_QUEUE_NAME.to_string());

        Ok(Self { temp_dir, env_vars })
    }

    pub fn set_env_vars(&self) {
        for (key, value) in &self.env_vars {
            std::env::set_var(key, value);
        }
    }

    pub fn cleanup(&self) {
        for key in self.env_vars.keys() {
            std::env::remove_var(key);
        }
    }
}

/// Async test timeout wrapper
pub async fn with_timeout<F, T>(future: F) -> Result<T, tokio::time::error::Elapsed>
where
    F: std::future::Future<Output = T>,
{
    timeout(TEST_TIMEOUT, future).await
}

/// Test assertion helpers
pub fn assert_message_equal(actual: &TestMessage, expected: &TestMessage) {
    pretty_assertions::assert_eq!(actual.producer_id, expected.producer_id);
    pretty_assertions::assert_eq!(actual.task_id, expected.task_id);
    pretty_assertions::assert_eq!(actual.message, expected.message);
}

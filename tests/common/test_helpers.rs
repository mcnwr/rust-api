use crate::common::TestMessage;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

/// HTTP test helpers
pub mod http {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};

    /// Create a test HTTP request
    pub fn create_test_request(method: &str, uri: &str, body: Option<String>) -> Request<Body> {
        let builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json");

        if let Some(body_content) = body {
            builder.body(Body::from(body_content)).unwrap()
        } else {
            builder.body(Body::empty()).unwrap()
        }
    }

    /// Assert HTTP response status
    pub fn assert_status_code(actual: StatusCode, expected: StatusCode) {
        pretty_assertions::assert_eq!(
            actual,
            expected,
            "Expected status code {}, got {}",
            expected.as_u16(),
            actual.as_u16()
        );
    }
}

/// Message testing utilities
pub mod messages {
    use super::*;

    /// Create test messages for producer testing
    pub fn create_test_messages(count: usize, producer_id: u32) -> Vec<TestMessage> {
        (0..count)
            .map(|i| {
                TestMessage::new(
                    producer_id,
                    i as u32,
                    format!("Test message {} from producer {}", i, producer_id),
                )
            })
            .collect()
    }

    /// Create JSON payload for publisher endpoint
    pub fn create_publisher_payload(producer_count: u32, message_count: u32) -> Value {
        serde_json::json!({
            "producer_count": producer_count,
            "message_count": message_count
        })
    }

    /// Validate message format
    pub fn validate_message_format(message_str: &str) -> Result<TestMessage, serde_json::Error> {
        let value: Value = serde_json::from_str(message_str)?;

        Ok(TestMessage {
            producer_id: value["producer_id"].as_u64().unwrap_or(0) as u32,
            task_id: value["task_id"].as_u64().unwrap_or(0) as u32,
            message: value["message"].as_str().unwrap_or("").to_string(),
        })
    }
}

/// Async test utilities
pub mod async_utils {
    use super::*;

    /// Wait for a condition to be true with timeout
    pub async fn wait_for_condition<F>(
        mut condition: F,
        timeout_duration: Duration,
        check_interval: Duration,
    ) -> bool
    where
        F: FnMut() -> bool,
    {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout_duration {
            if condition() {
                return true;
            }
            sleep(check_interval).await;
        }

        false
    }

    /// Retry an async operation with exponential backoff
    pub async fn retry_with_backoff<F, T, E>(
        mut operation: F,
        max_retries: usize,
        initial_delay: Duration,
    ) -> Result<T, E>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    {
        let mut delay = initial_delay;

        for attempt in 0..max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == max_retries - 1 {
                        return Err(e);
                    }
                    sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }

        unreachable!()
    }
}

/// Performance testing utilities
pub mod performance {
    use super::*;
    use std::time::Instant;

    /// Measure execution time of an async operation
    pub async fn measure_async<F, T>(operation: F) -> (T, Duration)
    where
        F: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = operation.await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Performance assertion helpers
    pub fn assert_duration_under(actual: Duration, max_duration: Duration) {
        assert!(
            actual <= max_duration,
            "Operation took {:?}, expected under {:?}",
            actual,
            max_duration
        );
    }

    /// Throughput calculation
    pub fn calculate_throughput(operations: usize, duration: Duration) -> f64 {
        operations as f64 / duration.as_secs_f64()
    }
}

/// Test data generators
pub mod generators {
    use super::*;
    use std::collections::HashMap;

    /// Generate random test message
    pub fn random_test_message() -> TestMessage {
        TestMessage::new(
            fastrand::u32(1..100),
            fastrand::u32(1..1000),
            format!("Random test message {}", fastrand::u32(1..10000)),
        )
    }

    /// Generate producer configuration for testing
    pub fn create_test_producer_config(
        producer_count: u32,
        message_count: u32,
    ) -> HashMap<String, Value> {
        let mut config = HashMap::new();
        config.insert("producer_count".to_string(), Value::from(producer_count));
        config.insert("message_count".to_string(), Value::from(message_count));
        config.insert(
            "rabbitmq_url".to_string(),
            Value::from(crate::common::MOCK_RABBITMQ_URL),
        );
        config.insert(
            "queue_name".to_string(),
            Value::from(crate::common::TEST_QUEUE_NAME),
        );
        config
    }
}

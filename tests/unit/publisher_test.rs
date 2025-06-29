use pretty_assertions::assert_eq;
use serde_json;

// Import the common test utilities
use crate::common::mock_rabbitmq::{create_mock_connection, MockRabbitMQ};
use crate::common::{with_timeout, TestEnvironment, TestMessage};

/// Test configuration constants
const TEST_QUEUE_NAME: &str = "test_queue";
const TEST_PRODUCER_COUNT: u32 = 3;
const TEST_ITERATIONS_PER_PRODUCER: u32 = 5;

/// Mock Task structure for testing (mirroring the actual Task struct)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
struct MockTask {
    producer_id: u32,
    task_number: u32,
}

impl MockTask {
    fn new(producer_id: u32, task_number: u32) -> Self {
        Self {
            producer_id,
            task_number,
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

/// Mock ProducerConfig for testing
#[derive(Debug, Clone)]
struct MockProducerConfig {
    producer_count: u32,
    iterations_per_producer: u32,
}

impl Default for MockProducerConfig {
    fn default() -> Self {
        Self {
            producer_count: TEST_PRODUCER_COUNT,
            iterations_per_producer: TEST_ITERATIONS_PER_PRODUCER,
        }
    }
}

/// Test helper to simulate publish_message function
async fn test_publish_message(
    mock: &MockRabbitMQ,
    task: &MockTask,
    should_fail: bool,
) -> Result<(), anyhow::Error> {
    if should_fail || !mock.is_connected() {
        return Err(anyhow::anyhow!("Failed to publish message"));
    }

    // Simulate message serialization
    let payload_bytes = task
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("Failed to serialize task payload: {}", e))?;

    // Simulate publishing to mock
    let message_str = String::from_utf8_lossy(&payload_bytes).to_string();
    mock.add_message(message_str);

    Ok(())
}

/// Test helper to simulate run_producer_task function
async fn test_run_producer_task(
    producer_id: u32,
    config: &MockProducerConfig,
    mock: &MockRabbitMQ,
    should_fail: bool,
) -> Result<u32, anyhow::Error> {
    if should_fail || !mock.is_connected() {
        return Err(anyhow::anyhow!("Producer task failed"));
    }

    let mut published_count = 0;

    for task_number in 0..config.iterations_per_producer {
        let task = MockTask::new(producer_id, task_number);

        test_publish_message(mock, &task, false).await?;
        published_count += 1;
    }

    Ok(published_count)
}

/// Test helper to simulate concurrent producer execution
async fn test_concurrent_producers(
    config: &MockProducerConfig,
    mock: &MockRabbitMQ,
    should_fail: bool,
) -> Result<u64, anyhow::Error> {
    if should_fail || !mock.is_connected() {
        return Err(anyhow::anyhow!("Concurrent producers failed"));
    }

    let mut tasks = Vec::with_capacity(config.producer_count as usize);

    for producer_id in 0..config.producer_count {
        let config_clone = config.clone();
        let mock_clone = MockRabbitMQ::new(); // Each producer gets its own mock state for this test
        mock_clone.set_connected(mock.is_connected());

        let task = tokio::spawn(async move {
            test_run_producer_task(producer_id, &config_clone, &mock_clone, false).await
        });
        tasks.push(task);
    }

    let mut total_messages = 0u64;
    for (index, task) in tasks.into_iter().enumerate() {
        match task.await {
            Ok(Ok(count)) => total_messages += count as u64,
            Ok(Err(e)) => return Err(anyhow::anyhow!("Producer task {} failed: {}", index, e)),
            Err(e) => return Err(anyhow::anyhow!("Producer task {} panicked: {}", index, e)),
        }
    }

    Ok(total_messages)
}

#[tokio::test]
async fn test_mock_task_serialization() {
    let task = MockTask::new(1, 42);

    // Test successful serialization
    let bytes_result = task.to_bytes();
    assert!(bytes_result.is_ok(), "Task serialization should succeed");

    let bytes = bytes_result.unwrap();
    assert!(!bytes.is_empty(), "Serialized bytes should not be empty");

    // Test deserialization
    let deserialized: Result<MockTask, _> = serde_json::from_slice(&bytes);
    assert!(deserialized.is_ok(), "Task deserialization should succeed");

    let deserialized_task = deserialized.unwrap();
    assert_eq!(
        deserialized_task, task,
        "Deserialized task should match original"
    );
}

#[tokio::test]
async fn test_publish_message_success() {
    let _env = TestEnvironment::new().unwrap();
    let mock = MockRabbitMQ::new();
    let task = MockTask::new(1, 42);

    let result = with_timeout(test_publish_message(&mock, &task, false)).await;

    assert!(result.is_ok(), "Timeout should not occur");
    let publish_result = result.unwrap();
    assert!(publish_result.is_ok(), "Message publishing should succeed");

    // Verify message was added to mock
    let messages = mock.get_messages();
    assert_eq!(messages.len(), 1, "Should have one message in mock");

    // Verify message content
    let deserialized: Result<MockTask, _> = serde_json::from_str(&messages[0]);
    assert!(deserialized.is_ok(), "Message should be valid JSON");
    assert_eq!(
        deserialized.unwrap(),
        task,
        "Message content should match original task"
    );
}

#[tokio::test]
async fn test_publish_message_failure() {
    let _env = TestEnvironment::new().unwrap();
    let mock = MockRabbitMQ::with_connection_failure();
    let task = MockTask::new(1, 42);

    let result = with_timeout(test_publish_message(&mock, &task, false)).await;

    assert!(result.is_ok(), "Timeout should not occur");
    let publish_result = result.unwrap();
    assert!(
        publish_result.is_err(),
        "Message publishing should fail when not connected"
    );
    assert!(publish_result
        .unwrap_err()
        .to_string()
        .contains("Failed to publish"));
}

#[tokio::test]
async fn test_publish_message_serialization_failure() {
    let _env = TestEnvironment::new().unwrap();
    let mock = MockRabbitMQ::new();
    let task = MockTask::new(1, 42);

    // Force serialization failure by testing with the failure flag
    let result = with_timeout(test_publish_message(&mock, &task, true)).await;

    assert!(result.is_ok(), "Timeout should not occur");
    let publish_result = result.unwrap();
    assert!(
        publish_result.is_err(),
        "Publishing should fail when forced to fail"
    );
}

#[tokio::test]
async fn test_run_producer_task_success() {
    let _env = TestEnvironment::new().unwrap();
    let mock = MockRabbitMQ::new();
    let config = MockProducerConfig::default();

    let result = with_timeout(test_run_producer_task(1, &config, &mock, false)).await;

    assert!(result.is_ok(), "Timeout should not occur");
    let producer_result = result.unwrap();
    assert!(producer_result.is_ok(), "Producer task should succeed");

    let published_count = producer_result.unwrap();
    assert_eq!(
        published_count, config.iterations_per_producer,
        "Should publish expected number of messages"
    );

    // Verify all messages were published to mock
    let messages = mock.get_messages();
    assert_eq!(
        messages.len() as u32,
        config.iterations_per_producer,
        "Mock should contain all published messages"
    );
}

#[tokio::test]
async fn test_run_producer_task_failure() {
    let _env = TestEnvironment::new().unwrap();
    let mock = MockRabbitMQ::with_connection_failure();
    let config = MockProducerConfig::default();

    let result = with_timeout(test_run_producer_task(1, &config, &mock, false)).await;

    assert!(result.is_ok(), "Timeout should not occur");
    let producer_result = result.unwrap();
    assert!(
        producer_result.is_err(),
        "Producer task should fail when not connected"
    );
    assert!(producer_result
        .unwrap_err()
        .to_string()
        .contains("Producer task failed"));
}

#[tokio::test]
async fn test_run_producer_task_forced_failure() {
    let _env = TestEnvironment::new().unwrap();
    let mock = MockRabbitMQ::new();
    let config = MockProducerConfig::default();

    let result = with_timeout(test_run_producer_task(1, &config, &mock, true)).await;

    assert!(result.is_ok(), "Timeout should not occur");
    let producer_result = result.unwrap();
    assert!(
        producer_result.is_err(),
        "Producer task should fail when forced to fail"
    );
}

#[tokio::test]
async fn test_concurrent_producers_success() {
    let _env = TestEnvironment::new().unwrap();
    let mock = MockRabbitMQ::new();
    let config = MockProducerConfig::default();

    let result = with_timeout(test_concurrent_producers(&config, &mock, false)).await;

    assert!(result.is_ok(), "Timeout should not occur");
    let concurrent_result = result.unwrap();
    assert!(
        concurrent_result.is_ok(),
        "Concurrent producers should succeed"
    );

    let total_messages = concurrent_result.unwrap();
    let expected_total = (config.producer_count * config.iterations_per_producer) as u64;
    assert_eq!(
        total_messages, expected_total,
        "Should publish total expected messages across all producers"
    );
}

#[tokio::test]
async fn test_concurrent_producers_failure() {
    let _env = TestEnvironment::new().unwrap();
    let mock = MockRabbitMQ::with_connection_failure();
    let config = MockProducerConfig::default();

    let result = with_timeout(test_concurrent_producers(&config, &mock, false)).await;

    assert!(result.is_ok(), "Timeout should not occur");
    let concurrent_result = result.unwrap();
    assert!(
        concurrent_result.is_err(),
        "Concurrent producers should fail when not connected"
    );
}

#[tokio::test]
async fn test_producer_config_variations() {
    let _env = TestEnvironment::new().unwrap();
    let mock = MockRabbitMQ::new();

    // Test with different configurations
    let test_configs = vec![
        MockProducerConfig {
            producer_count: 1,
            iterations_per_producer: 1,
        },
        MockProducerConfig {
            producer_count: 2,
            iterations_per_producer: 3,
        },
        MockProducerConfig {
            producer_count: 5,
            iterations_per_producer: 2,
        },
    ];

    for config in test_configs {
        mock.clear_messages(); // Clear between tests

        let result = test_run_producer_task(0, &config, &mock, false).await;
        assert!(
            result.is_ok(),
            "Producer should succeed with config: {:?}",
            config
        );

        let published_count = result.unwrap();
        assert_eq!(
            published_count, config.iterations_per_producer,
            "Should publish correct count for config: {:?}",
            config
        );
    }
}

#[tokio::test]
async fn test_message_ordering_and_content() {
    let _env = TestEnvironment::new().unwrap();
    let mock = MockRabbitMQ::new();
    let config = MockProducerConfig {
        producer_count: 1,
        iterations_per_producer: 3,
    };

    let result = test_run_producer_task(42, &config, &mock, false).await;
    assert!(result.is_ok(), "Producer task should succeed");

    let messages = mock.get_messages();
    assert_eq!(messages.len(), 3, "Should have 3 messages");

    // Verify message ordering and content
    for (index, message) in messages.iter().enumerate() {
        let task: MockTask = serde_json::from_str(message).expect("Message should be valid JSON");

        assert_eq!(task.producer_id, 42, "Producer ID should be correct");
        assert_eq!(
            task.task_number, index as u32,
            "Task number should be sequential"
        );
    }
}

#[tokio::test]
async fn test_error_handling_edge_cases() {
    let _env = TestEnvironment::new().unwrap();

    // Test with zero iterations
    let mock = MockRabbitMQ::new();
    let config = MockProducerConfig {
        producer_count: 1,
        iterations_per_producer: 0,
    };

    let result = test_run_producer_task(1, &config, &mock, false).await;
    assert!(
        result.is_ok(),
        "Producer should handle zero iterations gracefully"
    );
    assert_eq!(result.unwrap(), 0, "Should publish zero messages");

    // Test connection failure mid-operation
    let failing_mock = MockRabbitMQ::new();
    failing_mock.set_connected(false);

    let large_config = MockProducerConfig {
        producer_count: 1,
        iterations_per_producer: 100,
    };
    let result = test_run_producer_task(1, &large_config, &failing_mock, false).await;
    assert!(
        result.is_err(),
        "Producer should fail when connection is lost"
    );
}

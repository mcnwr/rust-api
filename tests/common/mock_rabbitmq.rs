use futures_lite::StreamExt;
use lapin::{
    options::{BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Error as LapinError,
    Result as LapinResult,
};
use mockall::{mock, predicate::*};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

/// Mock connection state for testing
#[derive(Debug, Clone)]
pub struct MockConnectionState {
    pub is_connected: bool,
    pub messages: Vec<String>,
    pub connection_failures: u32,
}

impl Default for MockConnectionState {
    fn default() -> Self {
        Self {
            is_connected: true,
            messages: Vec::new(),
            connection_failures: 0,
        }
    }
}

/// Mock RabbitMQ connection for testing
#[derive(Debug)]
pub struct MockRabbitMQ {
    pub state: Arc<Mutex<MockConnectionState>>,
}

impl MockRabbitMQ {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockConnectionState::default())),
        }
    }

    pub fn with_connection_failure() -> Self {
        let mut state = MockConnectionState::default();
        state.is_connected = false;
        state.connection_failures = 1;

        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub fn add_message(&self, message: String) {
        let mut state = self.state.lock().unwrap();
        state.messages.push(message);
    }

    pub fn get_messages(&self) -> Vec<String> {
        let state = self.state.lock().unwrap();
        state.messages.clone()
    }

    pub fn set_connected(&self, connected: bool) {
        let mut state = self.state.lock().unwrap();
        state.is_connected = connected;
    }

    pub fn is_connected(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.is_connected
    }

    pub fn clear_messages(&self) {
        let mut state = self.state.lock().unwrap();
        state.messages.clear();
    }
}

/// Mock connection creation function
pub async fn create_mock_connection(mock: &MockRabbitMQ) -> Result<MockConnection, anyhow::Error> {
    if !mock.is_connected() {
        return Err(anyhow::anyhow!("Connection failed"));
    }

    Ok(MockConnection {
        state: mock.state.clone(),
    })
}

/// Mock connection wrapper
#[derive(Debug)]
pub struct MockConnection {
    pub state: Arc<Mutex<MockConnectionState>>,
}

impl MockConnection {
    pub async fn create_channel(&self) -> Result<MockChannel, anyhow::Error> {
        if !self.state.lock().unwrap().is_connected {
            return Err(anyhow::anyhow!("Channel creation failed"));
        }

        Ok(MockChannel {
            state: self.state.clone(),
        })
    }

    pub async fn close(&self) -> Result<(), anyhow::Error> {
        let mut state = self.state.lock().unwrap();
        state.is_connected = false;
        Ok(())
    }
}

/// Mock channel wrapper
#[derive(Debug)]
pub struct MockChannel {
    pub state: Arc<Mutex<MockConnectionState>>,
}

impl MockChannel {
    pub async fn queue_declare(
        &self,
        _queue_name: &str,
        _options: QueueDeclareOptions,
        _arguments: FieldTable,
    ) -> Result<(), anyhow::Error> {
        if !self.state.lock().unwrap().is_connected {
            return Err(anyhow::anyhow!("Queue declaration failed"));
        }
        Ok(())
    }

    pub async fn basic_publish(
        &self,
        _exchange: &str,
        _routing_key: &str,
        _options: BasicPublishOptions,
        payload: &[u8],
        _properties: BasicProperties,
    ) -> Result<(), anyhow::Error> {
        if !self.state.lock().unwrap().is_connected {
            return Err(anyhow::anyhow!("Publish failed"));
        }

        let message = String::from_utf8_lossy(payload).to_string();
        let mut state = self.state.lock().unwrap();
        state.messages.push(message);

        Ok(())
    }

    pub async fn basic_consume(
        &self,
        _queue_name: &str,
        _consumer_tag: &str,
        _options: BasicConsumeOptions,
        _arguments: FieldTable,
    ) -> Result<MockConsumer, anyhow::Error> {
        if !self.state.lock().unwrap().is_connected {
            return Err(anyhow::anyhow!("Consumer creation failed"));
        }

        Ok(MockConsumer {
            state: self.state.clone(),
        })
    }
}

/// Mock consumer for testing
#[derive(Debug)]
pub struct MockConsumer {
    pub state: Arc<Mutex<MockConnectionState>>,
}

impl MockConsumer {
    pub async fn next_message(&self) -> Option<MockDelivery> {
        let mut state = self.state.lock().unwrap();
        if let Some(message) = state.messages.pop() {
            Some(MockDelivery {
                payload: message.into_bytes(),
                delivery_tag: 1,
            })
        } else {
            None
        }
    }
}

/// Mock delivery for testing
#[derive(Debug)]
pub struct MockDelivery {
    pub payload: Vec<u8>,
    pub delivery_tag: u64,
}

impl MockDelivery {
    pub fn data(&self) -> &[u8] {
        &self.payload
    }

    pub async fn ack(&self) -> Result<(), anyhow::Error> {
        // Mock acknowledgment - always succeeds
        Ok(())
    }
}

/// Test helper to create a populated mock
pub fn create_test_mock_with_messages(messages: Vec<&str>) -> MockRabbitMQ {
    let mock = MockRabbitMQ::new();
    for message in messages {
        mock.add_message(message.to_string());
    }
    mock
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use futures_lite::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::timeout;

const RABBITMQ_ADDRS: &str = "amqp://guest:guest@127.0.0.1:5672";
const QUEUE_NAME: &str = "test";
const PRODUCER_COUNT: u32 = 10;
const ITERATION_PER_PRODUCER: u32 = 100000;
const CONSUMER_TAG: &str = "my_consumer";
const EMPTY_QUEUE_TIMEOUT: u64 = 10;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    producer_id: u32,
    task_number: u32,
}

#[derive(Debug, Clone)]
struct ProducerConfig {
    producer_count: u32,
    iterations_per_producer: u32,
}

impl Default for ProducerConfig {
    fn default() -> Self {
        Self {
            producer_count: PRODUCER_COUNT,
            iterations_per_producer: ITERATION_PER_PRODUCER,
        }
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn create_connection() -> Result<Connection> {
    Connection::connect(RABBITMQ_ADDRS, ConnectionProperties::default())
        .await
        .map_err(|e| format!("Failed to connect to RabbitMQ: {}", e).into())
}

async fn setup_channel_and_queue(conn: &Connection) -> Result<Channel> {
    let channel = conn
        .create_channel()
        .await
        .map_err(|e| format!("Failed to create channel: {}", e))?;

    channel
        .queue_declare(
            QUEUE_NAME,
            QueueDeclareOptions {
                durable: true,
                ..QueueDeclareOptions::default()
            },
            FieldTable::default(),
        )
        .await
        .map_err(|e| format!("Failed to declare queue '{}': {}", QUEUE_NAME, e))?;

    Ok(channel)
}

async fn publish_message(channel: &Channel, task: &Task) -> Result<()> {
    let payload_bytes =
        serde_json::to_vec(task).map_err(|e| format!("Failed to serialize task payload: {}", e))?;

    channel
        .basic_publish(
            "",
            QUEUE_NAME,
            BasicPublishOptions::default(),
            &payload_bytes,
            BasicProperties::default().with_delivery_mode(2),
        )
        .await
        .map_err(|e| format!("Failed to publish message: {}", e))?
        .await
        .map_err(|e| format!("Failed to confirm message delivery: {}", e))?;

    Ok(())
}

async fn run_producer_task(producer_id: u32, config: &ProducerConfig) -> Result<()> {
    let conn = create_connection().await?;
    let channel = setup_channel_and_queue(&conn).await?;

    for task_number in 0..config.iterations_per_producer {
        let task = Task {
            producer_id,
            task_number,
        };

        publish_message(&channel, &task).await?;
    }

    Ok(())
}

async fn setup_consumer() -> Result<(
    Connection,
    Channel,
    impl StreamExt<Item = lapin::Result<lapin::message::Delivery>>,
)> {
    let conn = create_connection().await?;
    let channel = setup_channel_and_queue(&conn).await?;

    let consumer = channel
        .basic_consume(
            QUEUE_NAME,
            CONSUMER_TAG,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| format!("Failed to start consumer: {}", e))?;

    Ok((conn, channel, consumer))
}

async fn close_connection_gracefully(conn: Connection, reason: &str) {
    if let Err(e) = conn.close(200, "Consumer finished").await {
        eprintln!("Warning: Failed to close connection gracefully: {}", e);
    } else {
        println!("Connection closed successfully: {}", reason);
    }
}

pub async fn publisher() -> Response {
    let start = Instant::now();
    println!("===== Starting RabbitMQ Producer =====");

    let config = ProducerConfig::default();
    let total_messages = (config.producer_count * config.iterations_per_producer) as u64;

    let mut tasks = Vec::with_capacity(config.producer_count as usize);

    for producer_id in 0..config.producer_count {
        let config_clone = config.clone();
        let task = tokio::spawn(async move { run_producer_task(producer_id, &config_clone).await });
        tasks.push(task);
    }

    for (index, task) in tasks.into_iter().enumerate() {
        if let Err(e) = task.await {
            let error_msg = format!("Producer task {} failed: {}", index, e);
            eprintln!("{}", error_msg);
            return (StatusCode::INTERNAL_SERVER_ERROR, error_msg).into_response();
        }
    }

    let elapsed = start.elapsed();
    let success_message = format!(
        "[PRODUCER] Successfully sent {} messages in {:?}",
        total_messages, elapsed
    );

    println!("{}", success_message);
    (StatusCode::OK, success_message).into_response()
}

pub async fn consumer() -> Response {
    let start = Instant::now();
    println!("===== Starting RabbitMQ Consumer (Auto-Close) =====");
    println!(
        "Will auto-close after {} seconds of no messages",
        EMPTY_QUEUE_TIMEOUT
    );

    let (conn, _channel, mut consumer) = match setup_consumer().await {
        Ok((conn, channel, consumer)) => (conn, channel, consumer),
        Err(e) => {
            let error_msg = format!("Failed to setup consumer: {}", e);
            eprintln!("{}", error_msg);
            return (StatusCode::INTERNAL_SERVER_ERROR, error_msg).into_response();
        }
    };

    let mut message_count = 0u64;
    let timeout_duration = Duration::from_secs(EMPTY_QUEUE_TIMEOUT);

    loop {
        match timeout(timeout_duration, consumer.next()).await {
            Ok(Some(delivery_result)) => match delivery_result {
                Ok(delivery) => {
                    message_count += 1;
                    println!("Processed message #{}", message_count);

                    if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                        let error_msg = format!("Failed to acknowledge message: {}", e);
                        eprintln!("{}", error_msg);
                        close_connection_gracefully(conn, "Error occurred").await;
                        return (StatusCode::INTERNAL_SERVER_ERROR, error_msg).into_response();
                    }
                }
                Err(e) => {
                    if e.to_string().contains("connection aborted") {
                        println!("Connection terminated by server");
                        break;
                    }

                    let error_msg = format!("Error receiving message: {}", e);
                    eprintln!("{}", error_msg);
                    close_connection_gracefully(conn, "Error occurred").await;
                    return (StatusCode::INTERNAL_SERVER_ERROR, error_msg).into_response();
                }
            },
            Ok(None) => {
                println!("Consumer stream ended");
                break;
            }
            Err(_) => {
                println!(
                    "No messages received for {} seconds - queue appears empty",
                    EMPTY_QUEUE_TIMEOUT
                );
                break;
            }
        }
    }

    close_connection_gracefully(conn, "Queue empty or processing complete").await;

    let elapsed = start.elapsed();
    let success_message = format!(
        "[CONSUMER] Processed {} messages in {:?} (auto-closed when queue empty)",
        message_count, elapsed
    );

    println!("{}", success_message);
    (StatusCode::OK, success_message).into_response()
}

use anyhow::Result;
use futures::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, ExchangeDeclareOptions,
        QueueBindOptions, QueueDeclareOptions,
    },
    types::FieldTable,
    Connection, ConnectionProperties, ExchangeKind,
};
use std::time::Duration;

use crate::config::Config;
use crate::db::{self, DbPool};
use crate::models::{extract_reading, TelemetryPayload};

pub async fn start_consumer(pool: DbPool, config: Config) -> Result<()> {
    loop {
        if let Err(e) = run_consumer(&pool, &config).await {
            tracing::error!("consumer caiu: {e}");
        }
        tracing::info!("reconectando consumer em 5s...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn run_consumer(pool: &DbPool, config: &Config) -> Result<()> {
    let conn = connect_with_retry(&config.rabbitmq_url).await?;
    let channel = conn.create_channel().await?;

    channel
        .exchange_declare(
            &config.rabbitmq_exchange,
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_declare(
            &config.rabbitmq_queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            &config.rabbitmq_queue,
            &config.rabbitmq_exchange,
            &config.rabbitmq_routing_key,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            &config.rabbitmq_queue,
            "telemetry-consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tracing::info!("consumer ouvindo na fila {}", config.rabbitmq_queue);

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => match process_delivery(pool, &delivery.data).await {
                Ok(_) => {
                    delivery.ack(BasicAckOptions::default()).await?;
                }
                Err(e) => {
                    tracing::error!("erro ao processar mensagem: {e}");
                    delivery
                        .nack(BasicNackOptions {
                            requeue: true,
                            ..Default::default()
                        })
                        .await?;
                }
            },
            Err(e) => {
                tracing::error!("erro no delivery: {e}");
                return Err(e.into());
            }
        }
    }
    Ok(())
}

async fn process_delivery(pool: &DbPool, data: &[u8]) -> Result<()> {
    let payload: TelemetryPayload = serde_json::from_slice(data)?;
    let reading = extract_reading(&payload).map_err(|e| anyhow::anyhow!(e))?;
    db::insert_reading(pool, &reading)?;
    Ok(())
}

async fn connect_with_retry(url: &str) -> Result<Connection> {
    for attempt in 1..=10 {
        match Connection::connect(url, ConnectionProperties::default()).await {
            Ok(conn) => {
                tracing::info!("consumer conectado ao RabbitMQ (tentativa {attempt})");
                return Ok(conn);
            }
            Err(e) => {
                tracing::warn!("tentativa {attempt}/10 falhou: {e}");
                if attempt == 10 {
                    return Err(e.into());
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
    unreachable!()
}

use anyhow::Result;
use lapin::{
    options::{
        BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use std::time::Duration;

use crate::config::Config;
use crate::models::TelemetryPayload;

pub struct Publisher {
    channel: Channel,
    exchange: String,
    routing_key: String,
}

impl Publisher {
    pub async fn connect(config: &Config) -> Result<Self> {
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

        tracing::info!("publisher conectado ao RabbitMQ");

        Ok(Self {
            channel,
            exchange: config.rabbitmq_exchange.clone(),
            routing_key: config.rabbitmq_routing_key.clone(),
        })
    }

    pub async fn publish(&self, payload: &TelemetryPayload) -> Result<()> {
        let body = serde_json::to_vec(payload)?;
        self.channel
            .basic_publish(
                &self.exchange,
                &self.routing_key,
                BasicPublishOptions::default(),
                &body,
                BasicProperties::default().with_delivery_mode(2),
            )
            .await?
            .await?;
        Ok(())
    }
}

async fn connect_with_retry(url: &str) -> Result<Connection> {
    for attempt in 1..=10 {
        match Connection::connect(url, ConnectionProperties::default()).await {
            Ok(conn) => {
                tracing::info!("conectado ao RabbitMQ (tentativa {attempt})");
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

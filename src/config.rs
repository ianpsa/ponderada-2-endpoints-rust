#[derive(Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub rabbitmq_url: String,
    pub rabbitmq_exchange: String,
    pub rabbitmq_queue: String,
    pub rabbitmq_routing_key: String,
    pub sqlite_path: String,
    pub db_pool_size: u32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            rabbitmq_url: std::env::var("RABBITMQ_URL")
                .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".to_string()),
            rabbitmq_exchange: std::env::var("RABBITMQ_EXCHANGE")
                .unwrap_or_else(|_| "telemetry".to_string()),
            rabbitmq_queue: std::env::var("RABBITMQ_QUEUE")
                .unwrap_or_else(|_| "telemetry.readings".to_string()),
            rabbitmq_routing_key: std::env::var("RABBITMQ_ROUTING_KEY")
                .unwrap_or_else(|_| "sensor.reading".to_string()),
            sqlite_path: std::env::var("SQLITE_PATH")
                .unwrap_or_else(|_| "./data/telemetry.db".to_string()),
            db_pool_size: std::env::var("DB_POOL_SIZE")
                .unwrap_or_else(|_| "5".to_string())
                .parse()?,
        })
    }
}

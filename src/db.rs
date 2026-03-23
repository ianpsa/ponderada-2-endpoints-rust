use anyhow::Result;
use r2d2_sqlite::SqliteConnectionManager;
use crate::models::SensorReading;

pub type DbPool = r2d2::Pool<SqliteConnectionManager>;

pub fn build_pool(path: &str, size: u32) -> Result<DbPool> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let manager = SqliteConnectionManager::file(path);
    let pool = r2d2::Pool::builder().max_size(size).build(manager)?;
    Ok(pool)
}

pub fn run_migrations(pool: &DbPool) -> Result<()> {
    let conn = pool.get()?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS telemetry (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            temperature REAL,
            humidity REAL,
            presence INTEGER,
            vibration REAL,
            luminosity REAL,
            level REAL
        );
        CREATE INDEX IF NOT EXISTS idx_device_timestamp ON telemetry(device_id, timestamp);",
    )?;
    Ok(())
}

pub fn insert_reading(pool: &DbPool, r: &SensorReading) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO telemetry (device_id, timestamp, temperature, humidity, presence, vibration, luminosity, level)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            r.device_id,
            r.timestamp.to_rfc3339(),
            r.temperature,
            r.humidity,
            r.presence,
            r.vibration,
            r.luminosity,
            r.level,
        ],
    )?;
    tracing::trace!("inserted reading to sqlite");
    Ok(())
}

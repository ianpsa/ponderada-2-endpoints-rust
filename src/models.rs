use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryPayload {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub sensor_type: SensorType,
    pub reading_nature: ReadingNature,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorType {
    Temperature,
    Humidity,
    Presence,
    Vibration,
    Luminosity,
    ReservoirLevel,
}

impl std::fmt::Display for SensorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorType::Temperature => write!(f, "temperature"),
            SensorType::Humidity => write!(f, "humidity"),
            SensorType::Presence => write!(f, "presence"),
            SensorType::Vibration => write!(f, "vibration"),
            SensorType::Luminosity => write!(f, "luminosity"),
            SensorType::ReservoirLevel => write!(f, "reservoir_level"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingNature {
    Discrete,
    Analog,
}

impl std::fmt::Display for ReadingNature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadingNature::Discrete => write!(f, "discrete"),
            ReadingNature::Analog => write!(f, "analog"),
        }
    }
}

pub struct SensorReading {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub temperature: Option<f64>,
    pub humidity: Option<f64>,
    pub presence: Option<i32>,   // 1 for present and 0 for absent
    pub vibration: Option<f64>,
    pub luminosity: Option<f64>,
    pub level: Option<f64>,
}

pub fn extract_reading(payload: &TelemetryPayload) -> Result<SensorReading, String> {
    let analog = payload.value.as_f64();
    let discrete = payload.value.as_bool().map(|b| if b { 1 } else { 0 });

    Ok(SensorReading {
        device_id: payload.device_id.clone(),
        timestamp: payload.timestamp,
        temperature: matches!(payload.sensor_type, SensorType::Temperature).then(||
            analog.ok_or("temperature value must be a number")).transpose()?,
        humidity: matches!(payload.sensor_type, SensorType::Humidity).then(||
            analog.ok_or("humidity value must be a number")).transpose()?,
        presence: matches!(payload.sensor_type, SensorType::Presence).then(||
            discrete.ok_or("presence value must be a boolean")).transpose()?,
        vibration: matches!(payload.sensor_type, SensorType::Vibration).then(||
            analog.ok_or("vibration value must be a number")).transpose()?,
        luminosity: matches!(payload.sensor_type, SensorType::Luminosity).then(||
            analog.ok_or("luminosity value must be a number")).transpose()?,
        level: matches!(payload.sensor_type, SensorType::ReservoirLevel).then(||
            analog.ok_or("level value must be a number")).transpose()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_luminosity() {
        let payload = TelemetryPayload {
            device_id: "test-device".to_string(),
            timestamp: Utc::now(),
            sensor_type: SensorType::Luminosity,
            reading_nature: ReadingNature::Analog,
            value: json!(75.5),
        };
        let reading = extract_reading(&payload).unwrap();
        assert_eq!(reading.luminosity, Some(75.5));
    }

    #[test]
    fn test_extract_presence() {
        let payload = TelemetryPayload {
            device_id: "test-device".to_string(),
            timestamp: Utc::now(),
            sensor_type: SensorType::Presence,
            reading_nature: ReadingNature::Discrete,
            value: json!(true),
        };
        let reading = extract_reading(&payload).unwrap();
        assert_eq!(reading.presence, Some(1));
    }
}

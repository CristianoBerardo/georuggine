use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type)]
pub enum UserState {
    Sconnesso,
    Fermo,
    InMovimento,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub lat: f64,
    pub lon: f64,
    pub timestamp: DateTime<Utc>,
}

// Un singolo punto registrato nello storico, utile per l'analisi del tragitto
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TrackPoint {
    pub id: Option<i64>,
    pub user_id: i64,
    pub lat: f64,
    pub lon: f64,
    pub timestamp: DateTime<Utc>,
}

// Risultato di un'interrogazione di analisi movimento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementStats {
    pub distance_km: f64,
    pub avg_speed_kmh: f64,
    pub moving_duration_secs: u64,
    pub paused_duration_secs: u64,
}

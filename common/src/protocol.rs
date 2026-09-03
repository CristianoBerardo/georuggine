use crate::models::{MovementStats, Position};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Cosa può mandare il CLIENT al server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Register {
        username: String,
        password: String,
    },
    Login {
        username: String,
        password: String,
    },
    PositionUpdate {
        position: Position,
    },
    ChatMessage {
        message: String,
        timestamp: DateTime<Utc>,
    },
    QueryStats {
        period: TimePeriod,
    },
}

// Cosa può mandare il SERVER al client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    AuthResult {
        success: bool,
        reason: Option<String>,
        timestamp: DateTime<Utc>,
    },
    BroadcastMessage {
        message: String,
        timestamp: DateTime<Utc>,
    },
    DirectMessage {
        message: String,
        timestamp: DateTime<Utc>,
    },
    StatsResult {
        stats: MovementStats,
        timestamp: DateTime<Utc>,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimePeriod {
    Today,
    ThisWeek,
    ThisMonth,
}

#[derive(Clone)]
pub enum AuthAction {
    Login,
    Register,
}

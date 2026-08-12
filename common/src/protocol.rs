use crate::models::{MovementStats, Position};
use serde::{Deserialize, Serialize};

// Cosa può mandare il CLIENT al server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Register { username: String, password: String },
    Login { username: String, password: String },
    PositionUpdate(Position),
    ChatMessage(String),
    QueryStats { period: TimePeriod },
}

// Cosa può mandare il SERVER al client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    AuthResult {
        success: bool,
        reason: Option<String>,
    },
    BroadcastMessage(String),
    DirectMessage(String),
    StatsResult(MovementStats),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimePeriod {
    Today,
    ThisWeek,
    ThisMonth,
}

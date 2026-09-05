use crate::movement_sim::MovementStatus;
use chrono::{DateTime, Utc};
use common::models::MovementStats;
use common::protocol::TimePeriod;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Panel {
    UserInfo,
    Movement,
    StatsPeriod,
    Stats,
    Broadcast,
    Chat,
    ChatInput,
}

#[derive(Debug, Clone)]
pub(crate) struct ChatEntry {
    pub(crate) from_me: bool,
    pub(crate) is_system: bool,
    pub(crate) text: String,
    pub(crate) timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub(crate) struct BroadcastEntry {
    pub(crate) text: String,
    pub(crate) timestamp: DateTime<Utc>,
}

pub(crate) struct App {
    pub(crate) username: String,
    pub(crate) focus: Panel,
    pub(crate) chat_log: Vec<ChatEntry>,
    pub(crate) chat_input: String,
    pub(crate) broadcast_log: Vec<BroadcastEntry>,
    pub(crate) selected_period: TimePeriod,
    pub(crate) stats_pending: bool,
    pub(crate) stats: Option<MovementStats>,
    pub(crate) stats_error: Option<String>,
    pub(crate) stats_timestamp: Option<DateTime<Utc>>,
    pub(crate) movement_status: MovementStatus,
    pub(crate) chat_scroll: u16,
    pub(crate) broadcast_scroll: u16,
    pub(crate) chat_input_scroll: u16,
}

impl App {
    pub(crate) fn new(username: String) -> Self {
        App {
            username: username,
            focus: Panel::UserInfo,
            chat_log: Vec::new(),
            chat_input: String::new(),
            broadcast_log: Vec::new(),
            selected_period: TimePeriod::Today,
            stats_pending: false,
            stats: None,
            stats_error: None,
            stats_timestamp: None,
            movement_status: MovementStatus::default(),
            chat_scroll: 0,
            broadcast_scroll: 0,
            chat_input_scroll: 0,
        }
    }
}

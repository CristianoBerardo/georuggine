#[allow(dead_code)]
use chrono::{DateTime, Utc};
use common::models::MovementStats;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Panel {
    Users,
    SelectUser,
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

#[derive(Debug, Clone)]
pub(crate) struct ConnectedUsers {
    pub(crate) connected_users: Vec<String>,
    pub index_selected: usize,
}

pub(crate) struct App {
    /// The list of all users
    pub(crate) users: Vec<String>,
    /// The list of connected users
    pub(crate) connected_users: ConnectedUsers,
    pub(crate) focus: Panel,
    pub(crate) chat_log: Vec<ChatEntry>,
    pub(crate) chat_input: String,
    pub(crate) broadcast_chat_input: String,
    pub(crate) selected_user: String,
    pub(crate) chat_scroll: u16,
    pub(crate) broadcast_scroll: u16,
}

impl App {
    pub(crate) fn new(users: Vec<String>) -> Self {
        App {
            users,
            connected_users: ConnectedUsers {
                connected_users: [
                    "alice".to_string(),
                    "bob".to_string(),
                    "charlie".to_string(),
                ]
                .to_vec(),
                index_selected: 0,
            },
            focus: Panel::Users,
            chat_log: Vec::new(),
            chat_input: String::new(),
            broadcast_chat_input: String::new(),
            selected_user: String::new(),
            chat_scroll: 0,
            broadcast_scroll: 0,
        }
    }
}

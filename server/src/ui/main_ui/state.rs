use chrono::{DateTime, Utc};

use crate::state::UserStatus;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Panel {
    Users,
    SelectUser,
    Broadcast,
    BroadcastChat,
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
pub(crate) struct UserChat {
    pub(crate) username: String,
    pub(crate) chat_log: Vec<ChatEntry>,
    pub(crate) unread_count: usize,
    // pub(crate) chat_scroll: u16,
}

#[derive(Debug, Clone)]
pub(crate) struct ConnectedUsers {
    pub(crate) connected_users: Vec<UserChat>,
    pub index_selected: Option<usize>, // None = nessuna selezione attiva
}

#[derive(Debug, Clone)]
pub(crate) struct Users {
    pub(crate) username: String,
    pub(crate) status: UserStatus,
}

pub(crate) struct App {
    pub(crate) users: Vec<Users>,
    pub(crate) connected_users: ConnectedUsers,
    pub(crate) focus: Panel,
    pub(crate) broadcast_log: Vec<BroadcastEntry>,
    pub(crate) chat_input: String,
    pub(crate) broadcast_chat_input: String,
    pub(crate) users_scroll: u16,
    pub(crate) chat_scroll: u16,
    pub(crate) broadcast_scroll: u16,
    pub(crate) chat_input_scroll: u16,
    pub(crate) broadcast_input_scroll: u16,
}

impl App {
    pub(crate) async fn new(state: &crate::state::AppState) -> Self {
        let user_status: Vec<Users> = state
            .user_status
            .read()
            .await
            .iter()
            .clone()
            .map(|(username, status)| Users {
                username: username.clone(),
                status: status.status.clone(),
            })
            .collect();

        App {
            users: user_status,
            connected_users: ConnectedUsers {
                connected_users: Vec::new(),
                index_selected: None,
            },
            focus: Panel::Users,
            broadcast_log: Vec::new(),
            chat_input: String::new(),
            broadcast_chat_input: String::new(),
            users_scroll: 0,
            chat_scroll: 0,
            broadcast_scroll: 0,
            chat_input_scroll: 0,
            broadcast_input_scroll: 0,
        }
    }
}

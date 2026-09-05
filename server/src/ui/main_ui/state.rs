use chrono::{DateTime, Utc};

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
pub (crate) struct UserChat {
    pub(crate) username: String,
    pub(crate) chat_log: Vec<ChatEntry>,
    pub(crate) chat_scroll: u16,
}

#[derive(Debug, Clone)]
pub(crate) struct ConnectedUsers {
    pub(crate) connected_users: Vec<UserChat>,
    pub index_selected: Option<usize>, // None = nessuna selezione attiva
}

pub(crate) struct App {
    pub(crate) users: Vec<String>,
    pub(crate) connected_users: ConnectedUsers,
    pub(crate) focus: Panel,
    pub(crate) broadcast_log: Vec<BroadcastEntry>,
    pub(crate) chat_input: String,
    pub(crate) broadcast_chat_input: String,
    pub(crate) is_broadcast_mode: bool,
    pub(crate) chat_scroll: u16,
    pub(crate) broadcast_scroll: u16,
}

impl App {
    pub(crate) async fn new(state: &crate::state::AppState) -> Self {
        App {
            users: state.user_status.read().await.keys().cloned().collect(),
            connected_users: ConnectedUsers {
                connected_users: Vec::new(),
                index_selected: None,
            },
            focus: Panel::Users,
            broadcast_log: Vec::new(),
            chat_input: String::new(),
            broadcast_chat_input: String::new(),
            is_broadcast_mode: false,
            chat_scroll: 0,
            broadcast_scroll: 0,
        }
    }
}

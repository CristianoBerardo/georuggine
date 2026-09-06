use crate::movement_sim::MovementStatus;
use chrono::{DateTime, Utc};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Panel {
    UserInfo,
    DeleteAccount,
    Movement,
    Broadcast,
    Chat,
    ChatInput,
    ErrorLog,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DeleteAccountStep {
    Idle,
    EnterPassword,
    Confirm,
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
pub(crate) struct ErrorEntry {
    pub(crate) text: String,
    pub(crate) timestamp: DateTime<Utc>,
}

pub(crate) struct App {
    pub(crate) username: String,
    pub(crate) focus: Panel,
    pub(crate) chat_log: Vec<ChatEntry>,
    pub(crate) chat_input: String,
    pub(crate) broadcast_log: Vec<BroadcastEntry>,
    pub(crate) error_log: Vec<ErrorEntry>,
    pub(crate) movement_status: MovementStatus,
    pub(crate) chat_scroll: u16,
    pub(crate) broadcast_scroll: u16,
    pub(crate) chat_input_scroll: u16,
    pub(crate) error_scroll: u16,
    pub(crate) delete_step: DeleteAccountStep,
    pub(crate) delete_password: String,
    pub(crate) delete_pending: bool,
    pub(crate) delete_error: Option<String>,
}

impl App {
    pub(crate) fn new(username: String) -> Self {
        App {
            username: username,
            focus: Panel::UserInfo,
            chat_log: Vec::new(),
            chat_input: String::new(),
            broadcast_log: Vec::new(),
            error_log: Vec::new(),
            movement_status: MovementStatus::default(),
            chat_scroll: 0,
            broadcast_scroll: 0,
            chat_input_scroll: 0,
            error_scroll: 0,
            delete_step: DeleteAccountStep::Idle,
            delete_password: String::new(),
            delete_pending: false,
            delete_error: None,
        }
    }
}

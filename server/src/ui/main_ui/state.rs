use chrono::{DateTime, Utc};
use common::models::MovementStats;
use common::protocol::TimePeriod;

use crate::state::UserStatus;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Panel {
    Users,
    SelectUser,
    Broadcast,
    BroadcastChat,
    Chat,
    ChatInput,
    StatsSelect,
    ErrorLog,
}

// Il riquadro statistiche si compila in due passi: prima si sceglie l'utente
// (tra tutti i registrati, anche non collegati), poi il periodo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum StatsStep {
    SelectUser,
    SelectPeriod,
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
    pub(crate) error_log: Vec<ErrorEntry>,
    pub(crate) chat_input: String,
    pub(crate) broadcast_chat_input: String,
    pub(crate) users_scroll: u16,
    pub(crate) chat_scroll: u16,
    pub(crate) broadcast_scroll: u16,
    pub(crate) chat_input_scroll: u16,
    pub(crate) broadcast_input_scroll: u16,
    pub(crate) error_scroll: u16,
    pub(crate) stats_step: StatsStep,
    pub(crate) stats_user_index: Option<usize>,
    pub(crate) stats_period: TimePeriod,
    pub(crate) stats_username: Option<String>,
    pub(crate) stats_result: Option<MovementStats>,
    pub(crate) stats_error: Option<String>,
    pub(crate) stats_timestamp: Option<DateTime<Utc>>,
}

impl App {
    pub(crate) async fn new(state: &crate::state::AppState) -> Self {
        let mut user_status: Vec<Users> = state
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
        // Rimette i nomi sempre nello stesso ordine
        user_status.sort_by(|a, b| a.username.cmp(&b.username));

        App {
            users: user_status,
            connected_users: ConnectedUsers {
                connected_users: Vec::new(),
                index_selected: None,
            },
            focus: Panel::Users,
            broadcast_log: Vec::new(),
            error_log: Vec::new(),
            chat_input: String::new(),
            broadcast_chat_input: String::new(),
            users_scroll: 0,
            chat_scroll: 0,
            broadcast_scroll: 0,
            chat_input_scroll: 0,
            broadcast_input_scroll: 0,
            error_scroll: 0,
            stats_step: StatsStep::SelectUser,
            stats_user_index: None,
            stats_period: TimePeriod::Today,
            stats_username: None,
            stats_result: None,
            stats_error: None,
            stats_timestamp: None,
        }
    }
}

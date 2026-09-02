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
    pub(crate) text: String,
}

pub(crate) struct App {
    pub(crate) username: String,
    pub(crate) focus: Panel,
    pub(crate) chat_log: Vec<ChatEntry>,
    pub(crate) chat_input: String,
    pub(crate) broadcast_log: Vec<String>,
    pub(crate) selected_period: TimePeriod,
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
        }
    }
}

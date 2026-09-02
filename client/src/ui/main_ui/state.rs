pub(crate) struct App {
    pub(crate) username: String,
    pub(crate) focus: Panel,
}

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

impl App {
    pub(crate) fn new(username: String) -> Self {
        App {
            username: username,
            focus: Panel::UserInfo,
        }
    }
}

use common::protocol::AuthAction;

pub(crate) enum Screen {
    ChooseAction,
    Login,
    Register,
}

pub(crate) enum Focus {
    Username,
    Password,
    ConfirmPassword,
}

pub(crate) struct App {
    pub(crate) screen: Screen,
    pub(crate) focus: Focus,
    pub(crate) selected_action: AuthAction,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) confirm_password: String,
    pub(crate) error_message: Option<String>,
    pub(crate) info_message: Option<String>,
    pub(crate) awaiting_response: bool,
}

impl App {
    pub(crate) fn new() -> Self {
        App {
            screen: Screen::ChooseAction,
            focus: Focus::Username,
            selected_action: AuthAction::Login,
            username: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            error_message: None,
            info_message: None,
            awaiting_response: false,
        }
    }

    pub(crate) fn note_failure(&mut self, message: String) {
        self.awaiting_response = false;
        self.error_message = Some(message);
    }
}

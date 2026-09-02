pub(crate) struct App {
    pub(crate) username: String,
}

impl App {
    pub(crate) fn new(username: String) -> Self {
        App { username }
    }
}

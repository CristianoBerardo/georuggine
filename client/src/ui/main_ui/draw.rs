use super::state::App;
use ratatui::Frame;
use ratatui::widgets::Paragraph;

impl App {
    pub(crate) fn draw(&self, frame: &mut Frame) {
        let text = format!("Benvenuto, {}! (Esc per uscire)", self.username);
        frame.render_widget(Paragraph::new(text), frame.area());
    }
}

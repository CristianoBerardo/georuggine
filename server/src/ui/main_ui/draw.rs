use super::state::App;
use crate::ui::size_control::size_too_small;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Styled};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

impl App {
    pub(crate) fn draw(&self, frame: &mut Frame) {
        if size_too_small(frame.area()) {
            let warning = Paragraph::new("La dimensione del terminale è troppo piccola. Ridimensiona il terminale per continuare.")
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).title("Attenzione"));
            frame.render_widget(warning, frame.area());
            return;
        }

        // root[0] AREA PRINCIPALE
        // root[1] AREA DI AIUTO
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .split(frame.area());

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(root[0]);

        let col1 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // utenti
                Constraint::Length(10), // utenti connessi
                Constraint::Length(5),  // input messaggi broadcast
            ])
            .split(columns[0]);

        let col2 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(20), // chat diretta
                Constraint::Length(5),  // input messaggi
            ])
            .split(columns[1]);

        self.draw_chat_input(
            frame,
            col1[2],
            matches!(self.focus, super::state::Panel::ChatInput),
        );

        self.draw_help_bar(frame, root[1]);

        self.draw_chat(
            frame,
            col2[0],
            matches!(self.focus, super::state::Panel::Chat),
        );

        self.draw_users(
            frame,
            col1[0],
            matches!(self.focus, super::state::Panel::Users),
        );

        self.draw_connected_users(
            frame,
            col1[1],
            matches!(self.focus, super::state::Panel::SelectUser),
        );

        self.draw_broadcast_input(
            frame,
            col2[1],
            matches!(self.focus, super::state::Panel::Broadcast),
        );
    }

    fn draw_users(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Utenti registrati")
            .border_style(border_style);

        let lines: Vec<ListItem> = self
            .users
            .iter()
            .map(|user| {
                let status_text = match user.status {
                    crate::state::UserStatus::Sconnesso => "[Sconnesso]",
                    crate::state::UserStatus::Fermo => "[Fermo]",
                    crate::state::UserStatus::InMovimento => "[In movimento]",
                    crate::state::UserStatus::Problema => "[Problema]",
                };
                let text = format!("{} {}", user.username, status_text);
                ListItem::new(text)
            })
            .collect();

        let list = List::new(lines).block(block);
        frame.render_widget(list, area);
    }

    fn draw_connected_users(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        let items = self
            .connected_users
            .connected_users
            .iter()
            .enumerate()
            .map(|(index, user)| {
                if Some(index) == self.connected_users.index_selected {
                    ListItem::new(user.username.clone()).style(style)
                } else {
                    ListItem::new(user.username.clone()).style(Style::default())
                }
            })
            .collect::<Vec<ListItem>>();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Selezione utenti collegati")
                .border_style(border_style),
        );

        frame.render_widget(list, area);
    }

    pub fn draw_chat_input(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Scrivi messaggio")
            .border_style(border_style);

        if self.connected_users.index_selected.is_none() {
            let paragraph = Paragraph::new("Seleziona un utente collegato...")
                .block(block)
                .wrap(Wrap { trim: false });

            frame.render_widget(paragraph, area);
            return;
        }

        let inner_width = area.width.saturating_sub(2);
        let visible_rows = area.height.saturating_sub(2);

        let total_lines = Self::wrapped_line_count(&self.chat_input, inner_width);
        let top_offset = Self::scroll_offset(total_lines, area.height, self.chat_scroll);

        let paragraph = Paragraph::new(self.chat_input.as_str())
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((top_offset, 0));
        frame.render_widget(paragraph, area);

        if focused {
            let (cursor_row, cursor_col) =
                Self::wrapped_cursor_position(&self.chat_input, inner_width);
            let screen_row = cursor_row as i32 - top_offset as i32;
            if screen_row >= 0 && screen_row < visible_rows as i32 {
                frame.set_cursor_position(Position::new(
                    area.x + 1 + cursor_col,
                    area.y + 1 + screen_row as u16,
                ));
            }
        }
    }

    fn draw_broadcast_input(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Scrivi messaggio Broadcast")
            .border_style(border_style);

        let inner_width = area.width.saturating_sub(2);
        let visible_rows = area.height.saturating_sub(2);

        let total_lines = Self::wrapped_line_count(&self.broadcast_chat_input, inner_width);
        let top_offset = Self::scroll_offset(total_lines, area.height, self.broadcast_scroll);

        let paragraph = Paragraph::new(self.broadcast_chat_input.as_str())
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((top_offset, 0));
        frame.render_widget(paragraph, area);

        if focused {
            let (cursor_row, cursor_col) =
                Self::wrapped_cursor_position(&self.broadcast_chat_input, inner_width);
            let screen_row = cursor_row as i32 - top_offset as i32;
            if screen_row >= 0 && screen_row < visible_rows as i32 {
                frame.set_cursor_position(Position::new(
                    area.x + 1 + cursor_col,
                    area.y + 1 + screen_row as u16,
                ));
            }
        }
    }

    fn draw_chat(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        if self.is_broadcast_mode {
            self.draw_broadcast(frame, area, border_style);
        } else {
            self.draw_direct_chat(frame, area, border_style);
        }
    }

    fn draw_broadcast(&self, frame: &mut Frame, area: Rect, border_style: Style) {
        let chat_with_user = "Broadcast chat".to_string();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(chat_with_user)
            .border_style(border_style);

        let lines: Vec<ratatui::text::Line> = self
            .broadcast_log
            .iter()
            .map(|e| {
                let time = e.timestamp.with_timezone(&chrono::Local).format("%H:%M:%S");
                let text = format!("[{}] [sistema] {}", time, e.text);

                ratatui::text::Line::styled(text, Style::default())
            })
            .collect();

        let top_offset = Self::scroll_offset(
            self.broadcast_log.len() as u16,
            area.height,
            self.chat_scroll,
        );

        let paragraph = Paragraph::new(lines).block(block).scroll((top_offset, 0));
        frame.render_widget(paragraph, area);
    }

    fn draw_direct_chat(&self, frame: &mut Frame, area: Rect, border_style: Style) {
        let chat_with_user = if self.connected_users.index_selected.is_none() {
            "Nessun utente collegato selezionato".to_string()
        } else {
            format!(
                "Chat con: {}",
                self.connected_users.connected_users
                    [self.connected_users.index_selected.unwrap_or(0)]
                .username
            )
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(chat_with_user)
            .border_style(border_style);

        let user_selected_index = self.connected_users.index_selected;

        if user_selected_index.is_none() || self.connected_users.connected_users.is_empty() {
            let paragraph = Paragraph::new("").block(block);
            frame.render_widget(paragraph, area);
            return;
        } else {
            let inner_width = area.width.saturating_sub(2);

            let select_chat_connected_user =
                self.connected_users.connected_users[user_selected_index.unwrap_or(0)].clone();

            let formatted: Vec<(String, Style)> = select_chat_connected_user
                .chat_log
                .iter()
                .map(|e| {
                    let time = e.timestamp.with_timezone(&chrono::Local).format("%H:%M:%S");
                    let text = if e.is_system {
                        format!("[{}] [sistema] {}", time, e.text)
                    } else if e.from_me {
                        format!("[{}] > {}", time, e.text)
                    } else {
                        format!("[{}] < {}", time, e.text)
                    };
                    let style = Style::default();
                    (text, style)
                })
                .collect();

            let total_lines: u16 = formatted
                .iter()
                .map(|(text, _)| Self::wrapped_line_count(text, inner_width))
                .sum();

            let top_offset = Self::scroll_offset(total_lines, area.height, self.chat_scroll);

            let lines: Vec<ratatui::text::Line> = formatted
                .into_iter()
                .map(|(text, style)| ratatui::text::Line::styled(text, style))
                .collect();

            let paragraph = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((top_offset, 0));
            frame.render_widget(paragraph, area);
        }
    }

    fn scroll_offset(total_lines: u16, area_height: u16, scroll_up: u16) -> u16 {
        let visible = area_height.saturating_sub(2); // meno le due righe di bordo
        let max_scroll = total_lines.saturating_sub(visible);
        let effective_scroll_up = scroll_up.min(max_scroll);
        max_scroll - effective_scroll_up
    }

    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let hint = match self.focus {
            super::state::Panel::SelectUser => "↑/↓: Selezione · Invio: Seleziona utente collegati",
            super::state::Panel::ChatInput | super::state::Panel::Broadcast => {
                "Digita il messaggio · Invio: invia"
            }
            super::state::Panel::Chat => "↑/↓: scorri lo storico",
            _ => "Sola lettura",
        };
        let text = format!("Tab/Backtab: cambia riquadro · {} · Esc: esci", hint);

        let block = Block::default().borders(Borders::ALL).title("Aiuto");
        frame.render_widget(Paragraph::new(text).block(block), area);
    }

    fn wrapped_line_count(text: &str, width: u16) -> u16 {
        if width == 0 {
            return 1;
        }
        let width = width as usize;
        text.split('\n')
            .map(|line| {
                let mut rows: u16 = 1;
                let mut current_len = 0;
                for word in line.split_whitespace() {
                    let word_len = word.chars().count();
                    if current_len == 0 {
                        current_len = word_len;
                    } else if current_len + 1 + word_len <= width {
                        current_len += 1 + word_len;
                    } else {
                        rows += 1;
                        current_len = word_len;
                    }
                }
                rows
            })
            .sum()
    }

    fn wrapped_cursor_position(text: &str, width: u16) -> (u16, u16) {
        if width == 0 {
            return (0, 0);
        }
        let width = width as usize;
        let mut row: u16 = 0;
        let mut current_len = 0usize;
        for word in text.split_whitespace() {
            let word_len = word.chars().count();
            if current_len == 0 {
                current_len = word_len;
            } else if current_len + 1 + word_len <= width {
                current_len += 1 + word_len;
            } else {
                row += 1;
                current_len = word_len;
            }
        }
        (row, current_len as u16)
    }
}

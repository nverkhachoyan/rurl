use crossterm::event::{Event, MouseEventKind};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::components::Component;

pub enum FooterAction {
    Noop,
    Focused(bool),
    StatusUpdated(String),
}

pub struct Footer {
    focused: bool,
    status: String,
    rect: Option<Rect>,
    mode: String,
}

impl Footer {
    pub fn new() -> Self {
        Footer {
            focused: false,
            status: String::from("Ready"),
            rect: None,
            mode: String::from("NORMAL"),
        }
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn set_mode(&mut self, mode: String) {
        self.mode = mode;
    }

    fn render_mode_indicator(&self, mode: &str, color: Color) -> Vec<Span<'static>> {
        vec![Span::styled(
            format!(" {} ", mode),
            Style::default()
                .fg(Color::Black)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        )]
    }

    fn render_command(&self, key: &str, desc: &str, color: Color) -> Vec<Span<'static>> {
        vec![
            Span::raw(" ".to_string()),
            Span::styled(
                format!(" {} ", key),
                Style::default()
                    .fg(Color::Black)
                    .bg(color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ".to_string()),
            Span::styled(desc.to_string(), Style::default().fg(Color::White)),
        ]
    }

    fn render_separator(&self) -> Vec<Span<'static>> {
        vec![Span::styled(
            "  ".to_string(),
            Style::default().fg(Color::DarkGray),
        )]
    }

    pub fn render_status(&self, mode: &str) -> Line<'static> {
        match mode {
            "NORMAL" => {
                let mut spans = self.render_mode_indicator("NORMAL", Color::Green);
                spans.extend(self.render_command("SPACE", "command mode", Color::LightBlue));
                Line::from(spans)
            }
            "COMMAND" => {
                let mut spans = self.render_mode_indicator("COMMAND", Color::LightBlue);
                spans.extend(self.render_command("t", "tab mode", Color::Yellow));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("c", "create project", Color::Magenta));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("q", "quit", Color::Red));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("n", "normal mode", Color::Green));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("SPACE", "toggle mode", Color::LightBlue));
                Line::from(spans)
            }
            "TAB" => {
                let mut spans = self.render_mode_indicator("TAB", Color::Yellow);
                spans.extend(self.render_command("h/l", "switch tabs", Color::Cyan));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("1-9", "select tab", Color::LightBlue));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("d", "delete project", Color::Red));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("ESC", "back", Color::Red));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("SPACE", "show commands", Color::LightBlue));
                Line::from(spans)
            }
            "CREATE" => {
                let mut spans = self.render_mode_indicator("CREATE", Color::Magenta);
                spans.extend(self.render_command("ENTER", "confirm", Color::Green));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("ESC", "cancel", Color::Red));
                Line::from(spans)
            }
            _ => Line::from(vec![Span::raw(self.status.clone())]),
        }
    }
}

impl Component for Footer {
    type Action = FooterAction;

    fn tick(&mut self, event: Option<&Event>, _: u32) -> Self::Action {
        if let Some(event) = event {
            if let Event::Mouse(mouse_event) = event {
                match mouse_event.kind {
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        if let Some(rect) = self.rect {
                            if self.is_mouse_over(mouse_event, &rect) {
                                self.focused = true;
                                return FooterAction::Focused(true);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        FooterAction::Noop
    }

    fn render(&mut self, frame: &mut Frame, rect: Rect) {
        self.rect = Some(rect);

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(Color::Rgb(16, 18, 24)));

        let inner_rect = block.inner(rect);
        frame.render_widget(block, rect);

        let status = Paragraph::new(self.render_status(&self.mode))
            .alignment(Alignment::Left)
            .style(Style::default().bg(Color::Rgb(16, 18, 24)));

        frame.render_widget(status, inner_rect);
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

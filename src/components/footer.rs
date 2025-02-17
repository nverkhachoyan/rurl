use crossterm::event::{Event, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::actions::Action;
use crate::components::Component;

pub enum FooterAction {
    Noop,
    Focused(bool),
    Render,
    StatusUpdated(String),
}

pub struct Footer {
    focused: bool,
    status: String,
    rect: Option<Rect>,
}

impl Footer {
    pub fn new() -> Self {
        Footer {
            focused: false,
            status: String::from("Ready"),
            rect: None,
        }
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    fn render_mode_indicator(&self, mode: &str, color: Color) -> Vec<Span<'static>> {
        vec![
            Span::styled(" ".to_string(), Style::default().bg(color)),
            Span::styled(
                format!(" {} ", mode),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]
    }

    fn render_command(&self, key: &str, desc: &str, color: Color) -> Vec<Span<'static>> {
        vec![
            Span::raw(" ".to_string()),
            Span::styled(
                key.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(": ".to_string(), Style::default().fg(Color::DarkGray)),
            Span::styled(desc.to_string(), Style::default().fg(Color::White)),
        ]
    }

    fn render_separator(&self) -> Vec<Span<'static>> {
        vec![
            Span::raw(" ".to_string()),
            Span::styled("•".to_string(), Style::default().fg(Color::DarkGray)),
            Span::raw(" ".to_string()),
        ]
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
                spans.extend(self.render_command("c", "create project", Color::Cyan));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("q", "quit", Color::Red));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("n", "normal mode", Color::Green));
                Line::from(spans)
            }
            "TAB" => {
                let mut spans = self.render_mode_indicator("TAB", Color::Yellow);
                spans.extend(self.render_command("h/l", "switch tabs", Color::LightBlue));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("1-9", "select tab", Color::LightBlue));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("ESC", "back", Color::Red));
                spans.extend(self.render_separator());
                spans.extend(self.render_command("SPACE", "show commands", Color::Cyan));
                Line::from(spans)
            }
            "CREATE" => {
                let mut spans = self.render_mode_indicator("CREATE", Color::Cyan);
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

    fn tick(&mut self, event: Option<&Event>, tick_count: u32) -> Self::Action {
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
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if self.focused {
                Style::default().fg(Color::LightYellow)
            } else {
                Style::default().fg(Color::DarkGray)
            });

        let inner_rect = block.inner(rect);
        frame.render_widget(block, rect);

        let status = Paragraph::new(self.status.clone())
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White));

        frame.render_widget(status, inner_rect);
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

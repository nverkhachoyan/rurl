use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::components::Component;
use crate::persistence::RequestData;
pub enum ContentAction {
    Noop,
    Focused(bool),
    ContentUpdated,
}

pub struct Content {
    focused: bool,
    rect: Option<Rect>,
    request: Option<RequestData>,
}

impl Content {
    pub fn new() -> Self {
        Content {
            focused: false,
            rect: None,
            request: None,
        }
    }

    pub fn set_request(&mut self, request: RequestData) {
        self.request = Some(request);
    }

    pub fn clear_request(&mut self) {
        self.request = None;
    }
}

impl Component for Content {
    type Action = ContentAction;

    fn tick(&mut self, event: Option<&Event>, _: u32) -> Self::Action {
        if let Some(event) = event {
            match event {
                Event::Key(key_event) if self.focused => {
                    match key_event.code {
                        KeyCode::Char('j') | KeyCode::Down => {
                            // TODO: Implement scrolling down
                            return ContentAction::Noop;
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            // TODO: Implement scrolling up
                            return ContentAction::Noop;
                        }
                        _ => return ContentAction::Noop,
                    }
                }
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        if let Some(rect) = self.rect {
                            if self.is_mouse_over(mouse_event, &rect) {
                                self.focused = true;
                                return ContentAction::Focused(true);
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        ContentAction::Noop
    }

    fn render(&mut self, frame: &mut Frame, rect: Rect) {
        self.rect = Some(rect);
        let request_name = self
            .request
            .as_ref()
            .map_or("".to_string(), |r| r.url.clone().unwrap_or("".to_string()));

        let block = Block::default()
            .style(Style::default().bg(Color::Rgb(16, 18, 24))) // Base background
            .title(Span::styled(
                if self.focused {
                    format!(" ⦿ {}", request_name) // Filled circle for focused
                } else {
                    format!(" ○ {}", request_name) // Empty circle for unfocused
                },
                Style::default()
                    .fg(if self.focused {
                        Color::LightBlue
                    } else {
                        Color::Gray // Changed from DarkGray to Gray for better visibility
                    })
                    .add_modifier(Modifier::BOLD),
            ));

        let inner_rect = block.inner(rect);
        frame.render_widget(block, rect);

        if let Some(request) = &self.request {
            let method_style = match &request.method {
                Some(method) => match method.as_str() {
                    "GET" => Style::default().fg(Color::Green),
                    "POST" => Style::default().fg(Color::Blue),
                    "PUT" => Style::default().fg(Color::Yellow),
                    "DELETE" => Style::default().fg(Color::Red),
                    _ => Style::default().fg(Color::Gray),
                },
                None => Style::default().fg(Color::Gray),
            }
            .add_modifier(Modifier::BOLD);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // Method
                    Constraint::Length(2), // URL
                    Constraint::Length(2), // Headers
                    Constraint::Min(0),    // Future content (params, body, etc.)
                ])
                .split(inner_rect);

            // Method
            let method = Paragraph::new(Line::from(vec![
                Span::styled("Method:", Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled(
                    request.method.clone().unwrap_or("".to_string()),
                    method_style,
                ),
            ]))
            .style(Style::default().bg(Color::Rgb(16, 18, 24))); // Same background
            frame.render_widget(method, chunks[0]);

            // URL
            let url = Paragraph::new(Line::from(vec![
                Span::styled("URL:", Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled(
                    request.url.clone().unwrap_or("".to_string()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .style(Style::default().bg(Color::Rgb(16, 18, 24))); // Same background
            frame.render_widget(url, chunks[1]);

            // Headers
            let headers_string = request
                .headers
                .clone()
                .unwrap_or(vec![])
                .iter()
                .map(|(key, value)| format!("{}: {}", key, value))
                .collect::<Vec<String>>()
                .join("\n");

            let headers = Paragraph::new(Line::from(vec![
                Span::styled("Headers:", Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled(headers_string, Style::default().fg(Color::White)),
            ]))
            .style(Style::default().bg(Color::Rgb(16, 18, 24))); // Same background
            frame.render_widget(headers, chunks[2]);
        } else {
            let message = Paragraph::new(Line::from(vec![
                Span::styled(
                    "INFO:",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled("No request selected", Style::default().fg(Color::White)),
            ]))
            .alignment(Alignment::Center)
            .style(Style::default().bg(Color::Rgb(16, 18, 24))); // Same background
            frame.render_widget(message, inner_rect);
        }
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

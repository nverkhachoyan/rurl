use crossterm::event::{Event, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::components::{sidebar::ApiRequest, Component};

pub enum ContentAction {
    Noop,
    Focused(bool),
    Render,
    ContentUpdated,
}

pub struct Content {
    focused: bool,
    rect: Option<Rect>,
    request: Option<ApiRequest>,
}

impl Content {
    pub fn new() -> Self {
        Content {
            focused: false,
            rect: None,
            request: None,
        }
    }

    pub fn set_request(&mut self, request: ApiRequest) {
        self.request = Some(request);
    }
}

impl Component for Content {
    type Action = ContentAction;

    fn tick(&mut self, event: Option<&Event>, tick_count: u32) -> Self::Action {
        // Handle events
        if let Some(event) = event {
            if let Event::Mouse(mouse_event) = event {
                match mouse_event.kind {
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        if let Some(rect) = self.rect {
                            if self.is_mouse_over(mouse_event, &rect) {
                                self.focused = true;
                                return ContentAction::Focused(true);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        ContentAction::Noop
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
            })
            .title(Span::styled(
                " Request Details ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));

        let inner_rect = block.inner(rect);
        frame.render_widget(block, rect);

        if let Some(request) = &self.request {
            let method_style = match request.method.as_str() {
                "GET" => Style::default().fg(Color::Green),
                "POST" => Style::default().fg(Color::Blue),
                "PUT" => Style::default().fg(Color::Yellow),
                "DELETE" => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::Gray),
            }
            .add_modifier(Modifier::BOLD);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // Method
                    Constraint::Length(2), // Name
                    Constraint::Length(2), // URL
                    Constraint::Min(0),    // Future content (params, body, etc.)
                ])
                .split(inner_rect);

            // Method
            let method = Paragraph::new(Line::from(vec![
                Span::raw("Method: "),
                Span::styled(request.method.clone(), method_style),
            ]))
            .style(Style::default().fg(Color::White));
            frame.render_widget(method, chunks[0]);

            // Name
            let name = Paragraph::new(Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    request.name.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            frame.render_widget(name, chunks[1]);

            // URL
            let url = Paragraph::new(Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    request.url.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            frame.render_widget(url, chunks[2]);
        } else {
            let message = Paragraph::new("No request selected")
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center);
            frame.render_widget(message, inner_rect);
        }
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

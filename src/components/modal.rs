use crossterm::event::{Event, KeyCode, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::components::Component;
use crate::persistence::{AuthData, RequestData};

#[derive(PartialEq, Clone)]
pub enum ModalField {
    Name,
    Method,
    Url,
    Headers,
    QueryParams,
    PathParams,
    Auth,
    Body,
}

pub enum ModalAction {
    Noop,
    Close,
    Submit(RequestData),
}

pub struct Modal {
    focused: bool,
    rect: Option<Rect>,
    current_field: ModalField,
    fields: Vec<(ModalField, String)>,
    headers: Vec<(String, String)>,
    query_params: Vec<(String, String)>,
    path_params: Vec<(String, String)>,
    auth_type: AuthData,
}

impl Modal {
    pub fn new() -> Self {
        Modal {
            focused: true,
            rect: None,
            current_field: ModalField::Name,
            fields: vec![
                (ModalField::Name, String::new()),
                (ModalField::Method, String::new()),
                (ModalField::Url, String::new()),
                (ModalField::Headers, String::new()),
                (ModalField::QueryParams, String::new()),
                (ModalField::PathParams, String::new()),
                (ModalField::Auth, String::new()),
                (ModalField::Body, String::new()),
            ],
            headers: Vec::new(),
            query_params: Vec::new(),
            path_params: Vec::new(),
            auth_type: AuthData::None,
        }
    }

    fn get_previous_field(field: &ModalField) -> ModalField {
        match field {
            ModalField::Name => ModalField::Name,
            ModalField::Method => ModalField::Name,
            ModalField::Url => ModalField::Method,
            ModalField::Headers => ModalField::Url,
            ModalField::QueryParams => ModalField::Headers,
            ModalField::PathParams => ModalField::QueryParams,
            ModalField::Auth => ModalField::PathParams,
            ModalField::Body => ModalField::Auth,
        }
    }

    fn get_next_field(field: &ModalField) -> Option<ModalField> {
        match field {
            ModalField::Name => Some(ModalField::Method),
            ModalField::Method => Some(ModalField::Url),
            ModalField::Url => Some(ModalField::Headers),
            ModalField::Headers => Some(ModalField::QueryParams),
            ModalField::QueryParams => Some(ModalField::PathParams),
            ModalField::PathParams => Some(ModalField::Auth),
            ModalField::Auth => Some(ModalField::Body),
            ModalField::Body => None,
        }
    }

    fn handle_key_event(&mut self, key: KeyCode) -> ModalAction {
        match key {
            KeyCode::Esc => ModalAction::Close,
            KeyCode::Tab => {
                if let Some(next_field) = Self::get_next_field(&self.current_field) {
                    self.current_field = next_field;
                } else {
                    return self.submit_request();
                }
                ModalAction::Noop
            }
            KeyCode::BackTab => {
                self.current_field = Self::get_previous_field(&self.current_field);
                ModalAction::Noop
            }
            KeyCode::Enter => {
                if self.current_field == ModalField::Body {
                    return self.submit_request();
                }

                match self.current_field {
                    ModalField::Headers => {
                        if let Some((_, value)) = self
                            .fields
                            .iter_mut()
                            .find(|(field, _)| field == &ModalField::Headers)
                        {
                            if let Some((name, val)) = value.trim().split_once(':') {
                                self.headers
                                    .push((name.trim().to_string(), val.trim().to_string()));
                                value.clear();
                            }
                        }
                    }
                    ModalField::QueryParams => {
                        if let Some((_, value)) = self
                            .fields
                            .iter_mut()
                            .find(|(field, _)| field == &ModalField::QueryParams)
                        {
                            if let Some((name, val)) = value.trim().split_once('=') {
                                self.query_params
                                    .push((name.trim().to_string(), val.trim().to_string()));
                                value.clear();
                            }
                        }
                    }
                    ModalField::PathParams => {
                        if let Some((_, value)) = self
                            .fields
                            .iter_mut()
                            .find(|(field, _)| field == &ModalField::PathParams)
                        {
                            if let Some((name, val)) = value.trim().split_once('=') {
                                self.path_params
                                    .push((name.trim().to_string(), val.trim().to_string()));
                                value.clear();
                            }
                        }
                    }
                    ModalField::Auth => {
                        if let Some((_, value)) = self
                            .fields
                            .iter_mut()
                            .find(|(field, _)| field == &ModalField::Auth)
                        {
                            let parts: Vec<&str> = value.trim().split_whitespace().collect();
                            match parts.get(0).map(|s| *s) {
                                Some("basic") => {
                                    if parts.len() == 3 {
                                        self.auth_type = AuthData::Basic {
                                            username: parts[1].to_string(),
                                            password: parts[2].to_string(),
                                        };
                                    }
                                }
                                Some("bearer") => {
                                    if parts.len() == 2 {
                                        self.auth_type = AuthData::Bearer {
                                            token: parts[1].to_string(),
                                        };
                                    }
                                }
                                Some("apikey") => {
                                    if parts.len() == 4
                                        && (parts[3] == "header" || parts[3] == "query")
                                    {
                                        self.auth_type = AuthData::ApiKey {
                                            key: parts[1].to_string(),
                                            value: parts[2].to_string(),
                                            in_header: parts[3] == "header",
                                        };
                                    }
                                }
                                _ => self.auth_type = AuthData::None,
                            }
                            value.clear();
                        }
                    }
                    _ => {
                        if let Some(next_field) = Self::get_next_field(&self.current_field) {
                            self.current_field = next_field;
                        }
                    }
                }
                ModalAction::Noop
            }
            KeyCode::Backspace => {
                if let Some((_, value)) = self
                    .fields
                    .iter_mut()
                    .find(|(field, _)| field == &self.current_field)
                {
                    value.pop();
                }
                ModalAction::Noop
            }
            KeyCode::Char(c) => {
                if let Some((_, value)) = self
                    .fields
                    .iter_mut()
                    .find(|(field, _)| field == &self.current_field)
                {
                    value.push(c);
                }
                ModalAction::Noop
            }
            _ => ModalAction::Noop,
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: &MouseEvent, area: Rect) -> ModalAction {
        match mouse_event.kind {
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                let width = 60u16.min(area.width.saturating_sub(4));
                let height = 20u16.min(area.height.saturating_sub(4));
                let left = (area.width.saturating_sub(width)) / 2;
                let top = (area.height.saturating_sub(height)) / 2;
                let modal_area = Rect::new(left, top, width, height);

                let inner_area = Block::default().borders(Borders::ALL).inner(modal_area);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(vec![
                        Constraint::Length(3), // Name
                        Constraint::Length(3), // Method
                        Constraint::Length(3), // URL
                        Constraint::Length(3), // Headers
                        Constraint::Length(3), // Query Params
                        Constraint::Length(3), // Path Params
                        Constraint::Length(3), // Auth
                        Constraint::Length(3), // Body
                        Constraint::Length(1), // Help text
                    ])
                    .split(inner_area);

                for (i, (field, _)) in self.fields.iter().enumerate() {
                    let field_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(vec![
                            Constraint::Length(15), // Label
                            Constraint::Min(1),     // Input
                        ])
                        .split(chunks[i]);

                    let input_area = field_layout[1];
                    if mouse_event.row >= input_area.y
                        && mouse_event.row < input_area.y + input_area.height
                        && mouse_event.column >= input_area.x
                        && mouse_event.column < input_area.x + input_area.width
                    {
                        self.current_field = field.clone();
                        return ModalAction::Noop;
                    }
                }
            }
            _ => {}
        }
        ModalAction::Noop
    }

    fn submit_request(&self) -> ModalAction {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let name = self
            .fields
            .iter()
            .find(|(field, _)| field == &ModalField::Name)
            .map(|(_, v)| v.clone())
            .unwrap_or_default();

        let method = self
            .fields
            .iter()
            .find(|(field, _)| field == &ModalField::Method)
            .map(|(_, v)| v.clone());

        let url = self
            .fields
            .iter()
            .find(|(field, _)| field == &ModalField::Url)
            .map(|(_, v)| v.clone());

        let body = self
            .fields
            .iter()
            .find(|(field, _)| field == &ModalField::Body)
            .map(|(_, v)| v.clone());

        ModalAction::Submit(RequestData {
            name,
            method,
            url,
            headers: Some(self.headers.clone()),
            body,
            query_params: Some(self.query_params.clone()),
            path_params: Some(self.path_params.clone()),
            auth: Some(self.auth_type.clone()),
            created_at: now,
            updated_at: now,
        })
    }

    fn get_field_label(field: &ModalField) -> &'static str {
        match field {
            ModalField::Name => "Name:",
            ModalField::Method => "Method:",
            ModalField::Url => "URL:",
            ModalField::Headers => "Headers:",
            ModalField::QueryParams => "Query Params:",
            ModalField::PathParams => "Path Params:",
            ModalField::Auth => "Auth:",
            ModalField::Body => "Body:",
        }
    }
}

impl Component for Modal {
    type Action = ModalAction;

    fn tick(&mut self, event: Option<&Event>, _: u32) -> Self::Action {
        if let Some(event) = event {
            match event {
                Event::Key(key_event) => self.handle_key_event(key_event.code),
                Event::Mouse(mouse_event) => {
                    if let Some(rect) = self.rect {
                        self.handle_mouse_event(mouse_event, rect)
                    } else {
                        ModalAction::Noop
                    }
                }
                _ => ModalAction::Noop,
            }
        } else {
            ModalAction::Noop
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let width = 60u16.min(area.width.saturating_sub(4));
        let height = 30u16.min(area.height.saturating_sub(4));
        let left = (area.width.saturating_sub(width)) / 2;
        let top = (area.height.saturating_sub(height)) / 2;
        let modal_area = Rect::new(left, top, width, height);
        self.rect = Some(modal_area);

        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightYellow))
            .style(Style::default().bg(Color::DarkGray))
            .title(Span::styled(
                " New Request ",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ));

        frame.render_widget(block.clone(), modal_area);
        let inner_area = block.inner(modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(vec![
                Constraint::Length(3), // Name
                Constraint::Length(3), // Method
                Constraint::Length(3), // URL
                Constraint::Length(3), // Headers
                Constraint::Length(3), // Query Params
                Constraint::Length(3), // Path Params
                Constraint::Length(3), // Auth
                Constraint::Length(3), // Body
                Constraint::Length(1), // Help text
            ])
            .split(inner_area);

        for (i, (field, value)) in self.fields.iter().enumerate() {
            let is_current = &self.current_field == field;
            let field_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Length(15), // Label
                    Constraint::Min(1),     // Input
                ])
                .split(chunks[i]);

            // Label
            let label = Paragraph::new(Self::get_field_label(field)).style(Style::default().fg(
                if is_current {
                    Color::LightYellow
                } else {
                    Color::White
                },
            ));
            frame.render_widget(label, field_layout[0]);

            // Input with border
            let input_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if is_current {
                    Color::LightYellow
                } else {
                    Color::Gray
                }))
                .style(Style::default().bg(Color::Black));

            let input_area = input_block.inner(field_layout[1]);
            frame.render_widget(input_block, field_layout[1]);

            let input_style = if is_current {
                Style::default().fg(Color::LightYellow)
            } else {
                Style::default().fg(Color::White)
            };

            // special handling for fields with lists
            let text = match field {
                ModalField::Headers => {
                    let mut text = String::new();
                    for (name, val) in &self.headers {
                        text.push_str(&format!("{}: {}\n", name, val));
                    }
                    if is_current {
                        text.push_str(&format!("▎{}", value));
                    } else {
                        text.push_str(value);
                    }
                    text
                }
                ModalField::QueryParams => {
                    let mut text = String::new();
                    for (name, val) in &self.query_params {
                        text.push_str(&format!("{}={}\n", name, val));
                    }
                    if is_current {
                        text.push_str(&format!("▎{}", value));
                    } else {
                        text.push_str(value);
                    }
                    text
                }
                ModalField::PathParams => {
                    let mut text = String::new();
                    for (name, val) in &self.path_params {
                        text.push_str(&format!("{}={}\n", name, val));
                    }
                    if is_current {
                        text.push_str(&format!("▎{}", value));
                    } else {
                        text.push_str(value);
                    }
                    text
                }
                _ => {
                    if is_current {
                        format!("▎{}", value)
                    } else {
                        value.clone()
                    }
                }
            };

            let input = Paragraph::new(text).style(input_style);
            frame.render_widget(input, input_area);
        }

        let help_text =
            Paragraph::new("Tab/Enter: next field, Shift+Tab: previous field, Esc: cancel")
                .style(Style::default().fg(Color::White));
        frame.render_widget(help_text, chunks[8]);
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

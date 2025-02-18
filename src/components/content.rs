use crossterm::event::{Event, KeyCode, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::components::Component;
use crate::persistence::{RequestData, ResponseData};
use crate::theme::Theme;

#[derive(PartialEq)]
enum ViewMode {
    View,
    Edit,
}

#[derive(PartialEq, Clone)]
enum EditField {
    None,
    Method,
    Url,
    Headers,
    QueryParams,
    PathParams,
    Auth,
    Body,
}

pub enum ContentAction {
    Noop,
    ContentUpdated,
    RequestUpdated(RequestData),
}

pub struct Content {
    rect: Option<Rect>,
    request: Option<RequestData>,
    response: Option<ResponseData>,
    view_mode: ViewMode,
    edit_field: EditField,
    edit_buffer: String,
}

impl Content {
    pub fn new() -> Self {
        Content {
            rect: None,
            request: None,
            response: None,
            view_mode: ViewMode::View,
            edit_field: EditField::None,
            edit_buffer: String::new(),
        }
    }

    pub fn set_request(&mut self, request: RequestData) {
        self.request = Some(request);
        self.response = None;
        self.view_mode = ViewMode::View;
        self.edit_field = EditField::None;
        self.edit_buffer.clear();
    }

    pub fn clear_request(&mut self) {
        self.request = None;
        self.response = None;
    }

    pub fn enter_edit_mode(&mut self) {
        self.view_mode = ViewMode::Edit;
        self.edit_field = EditField::Method;
        if let Some(request) = &self.request {
            self.edit_buffer = request.method.clone().unwrap_or_default();
        }
    }

    fn get_next_field(field: &EditField) -> EditField {
        match field {
            EditField::None => EditField::Method,
            EditField::Method => EditField::Url,
            EditField::Url => EditField::Headers,
            EditField::Headers => EditField::QueryParams,
            EditField::QueryParams => EditField::PathParams,
            EditField::PathParams => EditField::Auth,
            EditField::Auth => EditField::Body,
            EditField::Body => EditField::Method,
        }
    }

    fn get_previous_field(field: &EditField) -> EditField {
        match field {
            EditField::None => EditField::Body,
            EditField::Method => EditField::Body,
            EditField::Url => EditField::Method,
            EditField::Headers => EditField::Url,
            EditField::QueryParams => EditField::Headers,
            EditField::PathParams => EditField::QueryParams,
            EditField::Auth => EditField::PathParams,
            EditField::Body => EditField::Auth,
        }
    }

    fn handle_edit_key(&mut self, key: KeyCode) -> ContentAction {
        if self.view_mode != ViewMode::Edit {
            return ContentAction::Noop;
        }

        match key {
            KeyCode::Esc => self.handle_escape_key(),
            KeyCode::Tab => self.handle_tab_key(false),
            KeyCode::BackTab => self.handle_tab_key(true),
            KeyCode::Enter => self.handle_enter_key(),
            KeyCode::Backspace => self.handle_backspace_key(),
            KeyCode::Char(c) => self.handle_char_key(c),
            _ => ContentAction::Noop,
        }
    }

    fn handle_escape_key(&mut self) -> ContentAction {
        if self.edit_field != EditField::None {
            self.edit_field = EditField::None;
            self.edit_buffer.clear();
            ContentAction::ContentUpdated
        } else {
            self.view_mode = ViewMode::View;
            ContentAction::ContentUpdated
        }
    }

    fn handle_tab_key(&mut self, backwards: bool) -> ContentAction {
        self.edit_field = if backwards {
            Self::get_previous_field(&self.edit_field)
        } else {
            Self::get_next_field(&self.edit_field)
        };

        if let Some(request) = &self.request {
            self.edit_buffer = match self.edit_field {
                EditField::Method => request.method.clone().unwrap_or_default(),
                EditField::Url => request.url.clone().unwrap_or_default(),
                EditField::Headers => String::new(),
                EditField::QueryParams => String::new(),
                EditField::PathParams => String::new(),
                EditField::Body => request.body.clone().unwrap_or_default(),
                _ => String::new(),
            };
        }
        ContentAction::ContentUpdated
    }

    fn handle_enter_key(&mut self) -> ContentAction {
        if let Some(mut request) = self.request.clone() {
            let action = match self.edit_field {
                EditField::Method => {
                    request.method = Some(self.edit_buffer.clone());
                    let action = ContentAction::RequestUpdated(request.clone());
                    self.request = Some(request);
                    self.edit_field = Self::get_next_field(&self.edit_field);
                    if let Some(req) = &self.request {
                        self.edit_buffer = req.url.clone().unwrap_or_default();
                    }
                    action
                }
                EditField::Url => {
                    request.url = Some(self.edit_buffer.clone());
                    let action = ContentAction::RequestUpdated(request.clone());
                    self.request = Some(request);
                    self.edit_field = Self::get_next_field(&self.edit_field);
                    self.edit_buffer.clear();
                    action
                }
                EditField::Headers => {
                    let action = self.handle_key_value_entry(&mut request.headers);
                    self.request = Some(request);
                    action
                }
                EditField::QueryParams => {
                    let action = self.handle_key_value_entry(&mut request.query_params);
                    self.request = Some(request);
                    action
                }
                EditField::PathParams => {
                    let action = self.handle_key_value_entry(&mut request.path_params);
                    self.request = Some(request);
                    action
                }
                EditField::Body => {
                    request.body = Some(self.edit_buffer.clone());
                    let action = ContentAction::RequestUpdated(request.clone());
                    self.request = Some(request);
                    self.edit_buffer.clear();
                    action
                }
                _ => ContentAction::Noop,
            };
            action
        } else {
            ContentAction::Noop
        }
    }

    fn handle_key_value_entry(
        &mut self,
        params: &mut Option<Vec<(String, String)>>,
    ) -> ContentAction {
        if let Some(request) = self.request.clone() {
            let separator = if self.edit_field == EditField::Headers {
                ':'
            } else {
                '='
            };
            if let Some((key, value)) = self.edit_buffer.split_once(separator) {
                let mut items = params.clone().unwrap_or_default();
                items.push((key.trim().to_string(), value.trim().to_string()));
                *params = Some(items);
                self.edit_buffer.clear();
                ContentAction::RequestUpdated(request)
            } else {
                ContentAction::ContentUpdated
            }
        } else {
            ContentAction::Noop
        }
    }

    fn handle_backspace_key(&mut self) -> ContentAction {
        if self.edit_field != EditField::None {
            self.edit_buffer.pop();
            ContentAction::ContentUpdated
        } else {
            ContentAction::Noop
        }
    }

    fn handle_char_key(&mut self, c: char) -> ContentAction {
        if self.edit_field != EditField::None {
            self.edit_buffer.push(c);
            ContentAction::ContentUpdated
        } else {
            ContentAction::Noop
        }
    }

    fn create_styled_block(&self, theme: &Theme, is_editing: bool) -> Block {
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if is_editing {
                theme.general.title_focused
            } else {
                theme.general.text_unfocused
            }))
            .style(Style::default().bg(theme.general.content_bg))
    }

    fn create_field_line<'a>(
        &self,
        icon: &'a str,
        title: &'a str,
        content: &'a str,
        theme: &'a Theme,
    ) -> Line<'a> {
        Line::from(vec![
            Span::styled(icon, Style::default().fg(theme.general.title_focused)),
            Span::raw(" "),
            Span::styled(
                title,
                Style::default()
                    .fg(theme.general.title_focused)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": "),
            Span::styled(content, Style::default().fg(theme.general.text)),
        ])
    }

    fn render_editable_field(
        &self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        icon: &str,
        content: String,
        is_editing: bool,
        theme: &Theme,
    ) {
        let content_text = if is_editing {
            format!("{}▎", self.edit_buffer)
        } else {
            content
        };

        let line = self.create_field_line(icon, title, &content_text, theme);
        let item = ListItem::new(line).style(Style::default().bg(if is_editing {
            theme.sidebar.selected_bg
        } else {
            theme.general.content_bg
        }));

        let list = List::new(vec![item]).block(self.create_styled_block(theme, is_editing));
        frame.render_widget(list, area);
    }

    fn get_method_style(&self, method: Option<&str>, theme: &Theme) -> Style {
        let color = match method {
            Some("GET") => theme.http_methods.get,
            Some("POST") => theme.http_methods.post,
            Some("PUT") => theme.http_methods.put,
            Some("DELETE") => theme.http_methods.delete,
            Some("PATCH") => theme.http_methods.patch,
            Some("HEAD") => theme.http_methods.head,
            _ => theme.http_methods.default,
        };
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    }

    fn get_status_style(&self, status_code: Option<i32>, theme: &Theme) -> Style {
        let code = status_code.unwrap_or(0);
        let color = match code {
            c if c >= 200 && c < 300 => theme.http_methods.get,
            c if c >= 300 && c < 400 => theme.http_methods.patch,
            c if c >= 400 && c < 500 => theme.http_methods.delete,
            c if c >= 500 => Color::Red,
            _ => theme.http_methods.default,
        };
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    }

    fn render_request_view(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(request) = &self.request {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(vec![
                    Constraint::Length(3), // Method
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // URL
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // Headers
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // Query Params
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // Path Params
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // Auth
                    Constraint::Length(1), // Spacer
                    Constraint::Min(4),    // Body
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // Submit button
                ])
                .split(area);

            // Render fields
            self.render_editable_field(
                frame,
                chunks[0],
                "Method",
                "→",
                request.method.clone().unwrap_or_default(),
                self.edit_field == EditField::Method,
                theme,
            );

            self.render_editable_field(
                frame,
                chunks[2],
                "URL",
                "🌐",
                request.url.clone().unwrap_or_default(),
                self.edit_field == EditField::Url,
                theme,
            );

            let headers_text = if self.edit_field == EditField::Headers {
                self.edit_buffer.clone()
            } else {
                self.format_key_value_list(&request.headers, ": ")
            };
            self.render_editable_field(
                frame,
                chunks[4],
                "Headers",
                "✉",
                headers_text,
                self.edit_field == EditField::Headers,
                theme,
            );

            let query_text = if self.edit_field == EditField::QueryParams {
                self.edit_buffer.clone()
            } else {
                self.format_key_value_list(&request.query_params, " = ")
            };
            self.render_editable_field(
                frame,
                chunks[6],
                "Query Parameters",
                "?",
                query_text,
                self.edit_field == EditField::QueryParams,
                theme,
            );

            let path_text = if self.edit_field == EditField::PathParams {
                self.edit_buffer.clone()
            } else {
                self.format_key_value_list(&request.path_params, " = ")
            };
            self.render_editable_field(
                frame,
                chunks[8],
                "Path Parameters",
                ":",
                path_text,
                self.edit_field == EditField::PathParams,
                theme,
            );

            let body_text = if self.edit_field == EditField::Body {
                self.edit_buffer.clone()
            } else {
                request.body.clone().unwrap_or_default()
            };
            self.render_editable_field(
                frame,
                chunks[12],
                "Body",
                "⚪",
                body_text,
                self.edit_field == EditField::Body,
                theme,
            );

            // Submit button
            let submit_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.http_methods.get))
                .style(Style::default().bg(theme.general.content_bg));

            let submit_text = "  Submit Changes  ";
            let submit_style = Style::default()
                .fg(theme.http_methods.get)
                .add_modifier(Modifier::BOLD);

            let submit_para = Paragraph::new(submit_text)
                .block(submit_block)
                .style(submit_style)
                .alignment(Alignment::Center);

            frame.render_widget(submit_para, chunks[14]);
        } else {
            self.render_empty_message(frame, area, "No request selected", theme);
        }
    }

    fn format_key_value_list(
        &self,
        items: &Option<Vec<(String, String)>>,
        separator: &str,
    ) -> String {
        items
            .as_ref()
            .map(|items| {
                items
                    .iter()
                    .map(|(k, v)| format!("{}{}{}", k, separator, v))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default()
    }

    fn render_empty_message(&self, frame: &mut Frame, area: Rect, message: &str, theme: &Theme) {
        let message = Paragraph::new(Line::from(vec![
            Span::styled(
                "ℹ",
                Style::default()
                    .fg(theme.general.title_focused)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(message, Style::default().fg(theme.general.text)),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().bg(theme.general.content_bg));
        frame.render_widget(message, area);
    }

    fn render_response_view(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(response) = &self.response {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3), // Status line
                    Constraint::Length(1), // Spacer
                    Constraint::Length(2), // Headers title
                    Constraint::Length(6), // Headers content
                    Constraint::Length(1), // Spacer
                    Constraint::Length(2), // Body title
                    Constraint::Min(4),    // Body content
                ])
                .split(area);

            // Status line
            let status_style = self.get_status_style(response.status_code, theme);
            let status_line = Line::from(vec![
                Span::styled(
                    format!(" {} ", response.status_code.unwrap_or(0)),
                    status_style,
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{}ms", response.response_time),
                    Style::default()
                        .fg(theme.general.text)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);

            let status_para = Paragraph::new(status_line)
                .block(self.create_styled_block(theme, false))
                .style(Style::default().bg(theme.general.content_bg));
            frame.render_widget(status_para, chunks[0]);

            // Headers
            let headers_title = self.create_field_line("✉", "Response Headers", "", theme);
            frame.render_widget(
                Paragraph::new(headers_title).style(Style::default().bg(theme.general.content_bg)),
                chunks[2],
            );

            let headers_content = response
                .response_headers
                .as_ref()
                .map(|headers| {
                    headers
                        .iter()
                        .map(|(key, value)| {
                            Line::from(vec![
                                Span::raw("  "),
                                Span::styled(
                                    format!("{}: ", key),
                                    Style::default()
                                        .fg(theme.general.text)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(value, Style::default().fg(theme.general.text)),
                            ])
                        })
                        .collect::<Vec<Line>>()
                })
                .unwrap_or_else(|| vec![Line::from(vec![Span::raw("  No headers")])]);

            let headers_para = Paragraph::new(headers_content).block(
                Block::default()
                    .style(Style::default().bg(theme.general.content_bg))
                    .borders(Borders::LEFT),
            );
            frame.render_widget(headers_para, chunks[3]);

            // Body
            let body_title = self.create_field_line("⚪", "Response Body", "", theme);
            frame.render_widget(
                Paragraph::new(body_title).style(Style::default().bg(theme.general.content_bg)),
                chunks[5],
            );

            let body_content = response
                .response_body
                .as_ref()
                .map(|body| body.as_str())
                .unwrap_or("No body");

            let body_para = Paragraph::new(body_content)
                .block(
                    Block::default()
                        .style(Style::default().bg(theme.general.content_bg))
                        .borders(Borders::LEFT),
                )
                .style(Style::default().fg(theme.general.text));
            frame.render_widget(body_para, chunks[6]);
        } else {
            self.render_empty_message(
                frame,
                area,
                "No response available - Send the request to see the response",
                theme,
            );
        }
    }

    fn render_request_summary(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(request) = &self.request {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3), // Request summary
                    Constraint::Length(1), // Spacer
                    Constraint::Min(4),    // Response
                ])
                .split(area);

            // Request summary
            let method_style = self.get_method_style(request.method.as_deref(), theme);
            let summary_line = Line::from(vec![
                Span::styled(
                    format!(" {} ", request.method.clone().unwrap_or_default()),
                    method_style,
                ),
                Span::raw(" "),
                Span::styled(
                    request.url.clone().unwrap_or_default(),
                    Style::default().fg(theme.general.text),
                ),
                Span::raw(" "),
                Span::styled(
                    "(Press 'e' to edit)",
                    Style::default().fg(theme.general.text_unfocused),
                ),
            ]);

            let summary_para = Paragraph::new(summary_line)
                .block(self.create_styled_block(theme, false))
                .style(Style::default().bg(theme.general.content_bg));
            frame.render_widget(summary_para, chunks[0]);

            // Response section
            if self.response.is_some() {
                self.render_response_view(frame, chunks[2], theme);
            } else {
                self.render_empty_message(frame, chunks[2], "No response available", theme);
            }
        } else {
            self.render_empty_message(frame, area, "No request selected", theme);
        }
    }

    fn handle_mouse_click(&mut self, mouse_event: &MouseEvent, area: Rect) -> ContentAction {
        if let Some(request) = &self.request {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(vec![
                    Constraint::Length(3), // Method
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // URL
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // Headers
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // Query Params
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // Path Params
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // Auth
                    Constraint::Length(1), // Spacer
                    Constraint::Min(4),    // Body
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // Submit button
                ])
                .split(area);

            let previous_field = self.edit_field.clone();

            // Helper function to check if mouse is within a rect's bounds
            let is_within = |rect: Rect| -> bool {
                mouse_event.row >= rect.y
                    && mouse_event.row < rect.y + rect.height
                    && mouse_event.column >= rect.x
                    && mouse_event.column < rect.x + rect.width
            };

            // Check which field was clicked
            self.edit_field = if is_within(chunks[0]) {
                EditField::Method
            } else if is_within(chunks[2]) {
                EditField::Url
            } else if is_within(chunks[4]) {
                EditField::Headers
            } else if is_within(chunks[6]) {
                EditField::QueryParams
            } else if is_within(chunks[8]) {
                EditField::PathParams
            } else if is_within(chunks[12]) {
                EditField::Body
            } else {
                self.edit_field.clone()
            };

            // If field changed, update edit buffer
            if self.edit_field != previous_field {
                self.edit_buffer = match self.edit_field {
                    EditField::Method => request.method.clone().unwrap_or_default(),
                    EditField::Url => request.url.clone().unwrap_or_default(),
                    EditField::Body => request.body.clone().unwrap_or_default(),
                    _ => String::new(),
                };
                return ContentAction::ContentUpdated;
            }

            // Check if submit button was clicked
            if is_within(chunks[14]) {
                if let Some(request) = self.request.clone() {
                    return ContentAction::RequestUpdated(request);
                }
            }
        }
        ContentAction::Noop
    }
}

impl Component for Content {
    type Action = ContentAction;

    fn tick(&mut self, event: Option<&Event>, _: u32) -> Self::Action {
        if let Some(event) = event {
            match event {
                Event::Key(key_event) => self.handle_edit_key(key_event.code),
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        if let Some(rect) = self.rect {
                            if self.is_mouse_over(mouse_event, &rect) {
                                self.handle_mouse_click(mouse_event, rect)
                            } else {
                                ContentAction::Noop
                            }
                        } else {
                            ContentAction::Noop
                        }
                    }
                    _ => ContentAction::Noop,
                },
                _ => ContentAction::Noop,
            }
        } else {
            ContentAction::Noop
        }
    }

    fn render(&mut self, frame: &mut Frame, rect: Rect, theme: &Theme) {
        self.rect = Some(rect);
        let request_name = self
            .request
            .as_ref()
            .map_or("".to_string(), |r| r.name.clone());

        let title_prefix = match self.view_mode {
            ViewMode::View => "View",
            ViewMode::Edit => "Edit",
        };

        let block = Block::default()
            .style(Style::default().bg(theme.general.content_bg))
            .title(Span::styled(
                format!(" {} - {} ", request_name, title_prefix),
                Style::default()
                    .fg(theme.general.title_focused)
                    .add_modifier(Modifier::BOLD),
            ));

        let inner_rect = block.inner(rect);
        frame.render_widget(block, rect);

        match self.view_mode {
            ViewMode::View => self.render_request_summary(frame, inner_rect, theme),
            ViewMode::Edit => self.render_request_view(frame, inner_rect, theme),
        }
    }
}

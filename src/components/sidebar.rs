use crate::components::Component;
use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

#[derive(Clone)]
pub struct ApiRequest {
    pub name: String,
    pub method: String,
    pub url: String,
}

pub enum SidebarAction {
    Noop,
    Focused(bool),
    Render,
    Selected(ApiRequest),
}

pub struct Sidebar {
    focused: bool,
    rect: Option<Rect>,
    requests: Vec<ApiRequest>,
    selected_index: Option<usize>,
}

impl Sidebar {
    pub fn new() -> Self {
        let example_requests = vec![
            ApiRequest {
                name: "Get Users".to_string(),
                method: "GET".to_string(),
                url: "https://api.example.com/users".to_string(),
            },
            ApiRequest {
                name: "Create User".to_string(),
                method: "POST".to_string(),
                url: "https://api.example.com/users".to_string(),
            },
            ApiRequest {
                name: "Update User".to_string(),
                method: "PUT".to_string(),
                url: "https://api.example.com/users/1".to_string(),
            },
        ];

        Sidebar {
            focused: false,
            rect: None,
            requests: example_requests,
            selected_index: None,
        }
    }

    pub fn set_requests(&mut self, requests: Vec<ApiRequest>) {
        self.requests = requests;
        self.selected_index = None;
    }

    pub fn clear_requests(&mut self) {
        self.requests.clear();
        self.selected_index = None;
    }

    fn handle_selection(&mut self, key: KeyCode) -> Option<SidebarAction> {
        match key {
            KeyCode::Char('j') | KeyCode::Down => {
                // move selection down
                let new_index = match self.selected_index {
                    Some(i) if i < self.requests.len() - 1 => Some(i + 1),
                    None if !self.requests.is_empty() => Some(0),
                    _ => self.selected_index,
                };
                self.selected_index = new_index;
                new_index.map(|i| SidebarAction::Selected(self.requests[i].clone()))
            }
            KeyCode::Char('k') | KeyCode::Up => {
                // move selection up
                let new_index = match self.selected_index {
                    Some(i) if i > 0 => Some(i - 1),
                    _ => self.selected_index,
                };
                self.selected_index = new_index;
                new_index.map(|i| SidebarAction::Selected(self.requests[i].clone()))
            }
            KeyCode::Enter => {
                // confirm selection
                self.selected_index
                    .map(|i| SidebarAction::Selected(self.requests[i].clone()))
            }
            _ => None,
        }
    }
}

impl Component for Sidebar {
    type Action = SidebarAction;

    fn tick(&mut self, event: Option<&Event>, tick_count: u32) -> Self::Action {
        if let Some(event) = event {
            match event {
                Event::Key(key_event) => {
                    if self.focused {
                        if let Some(action) = self.handle_selection(key_event.code) {
                            return action;
                        }
                    }
                }
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        if let Some(rect) = self.rect {
                            if self.is_mouse_over(mouse_event, &rect) {
                                self.focused = true;

                                // calculate which item was clicked, with bounds checking
                                if mouse_event.row <= rect.y {
                                    return SidebarAction::Focused(true);
                                }

                                let relative_y = mouse_event.row.saturating_sub(rect.y + 1); // +1 for border
                                if (relative_y as usize) < self.requests.len() {
                                    self.selected_index = Some(relative_y as usize);
                                    return SidebarAction::Selected(
                                        self.requests[relative_y as usize].clone(),
                                    );
                                }

                                return SidebarAction::Focused(true);
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        SidebarAction::Noop
    }

    fn render(&mut self, frame: &mut Frame, rect: Rect) {
        self.rect = Some(rect);

        let items: Vec<ListItem> = self
            .requests
            .iter()
            .enumerate()
            .map(|(i, request)| {
                let (method_style, name_style) = if Some(i) == self.selected_index {
                    (
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    (
                        match request.method.as_str() {
                            "GET" => Style::default().fg(Color::Green),
                            "POST" => Style::default().fg(Color::Blue),
                            "PUT" => Style::default().fg(Color::Yellow),
                            "DELETE" => Style::default().fg(Color::Red),
                            _ => Style::default().fg(Color::Gray),
                        },
                        Style::default().fg(Color::White),
                    )
                };

                let method = format!(" {} ", request.method);
                let name = format!(" {}", request.name);

                ListItem::new(Line::from(vec![
                    Span::styled(method, method_style),
                    Span::raw(" "),
                    Span::styled(name, name_style),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(if self.focused {
                        Style::default().fg(Color::LightYellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    })
                    .title(Span::styled(
                        " Requests ",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightYellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(list, rect);
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

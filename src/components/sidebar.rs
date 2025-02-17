use crate::components::Component;
use crate::persistence::{ProjectUpdate, RequestData};
use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
    Frame,
};

pub enum SidebarAction {
    Noop,
    Focused(bool),
    Selected(RequestData),
    ProjectUpdate(ProjectUpdate),
    ShowModal,
}

pub struct Sidebar {
    focused: bool,
    rect: Option<Rect>,
    requests: Vec<RequestData>,
    selected_index: Option<usize>,
}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {
            focused: false,
            rect: None,
            requests: vec![],
            selected_index: None,
        }
    }

    pub fn set_requests(&mut self, requests: Vec<RequestData>) {
        self.requests = requests;
        if self.requests.is_empty() {
            self.selected_index = None;
        } else {
            self.selected_index = Some(0);
        }
    }

    pub fn clear_requests(&mut self) {
        self.requests.clear();
        self.selected_index = None;
    }

    fn handle_selection(&mut self, key: KeyCode) -> Option<SidebarAction> {
        match key {
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.requests.is_empty() {
                    let new_index = match self.selected_index {
                        Some(i) if i < self.requests.len() - 1 => Some(i + 1),
                        None => Some(0),
                        _ => self.selected_index,
                    };
                    self.selected_index = new_index;
                    new_index.map(|i| SidebarAction::Selected(self.requests[i].clone()))
                } else {
                    None
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if !self.requests.is_empty() {
                    let new_index = match self.selected_index {
                        Some(i) if i > 0 => Some(i - 1),
                        None => Some(self.requests.len() - 1),
                        _ => self.selected_index,
                    };
                    self.selected_index = new_index;
                    new_index.map(|i| SidebarAction::Selected(self.requests[i].clone()))
                } else {
                    None
                }
            }
            KeyCode::Enter => self
                .selected_index
                .map(|i| SidebarAction::Selected(self.requests[i].clone())),
            KeyCode::Char('a') => Some(SidebarAction::ShowModal),
            KeyCode::Char('d') => self
                .selected_index
                .map(|i| SidebarAction::ProjectUpdate(ProjectUpdate::DeleteRequest(i))),
            _ => None,
        }
    }
}

impl Component for Sidebar {
    type Action = SidebarAction;

    fn tick(&mut self, event: Option<&Event>, _: u32) -> Self::Action {
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
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    (
                        if let Some(method) = &request.method {
                            match method.as_str() {
                                "GET" => Style::default().fg(Color::Green),
                                "POST" => Style::default().fg(Color::Blue),
                                "PUT" => Style::default().fg(Color::Yellow),
                                "DELETE" => Style::default().fg(Color::Red),
                                _ => Style::default().fg(Color::Gray),
                            }
                        } else {
                            Style::default().fg(Color::Gray)
                        }
                        .add_modifier(Modifier::BOLD),
                        Style::default().fg(Color::Gray),
                    )
                };

                let method = format!(" {} ", request.method.clone().unwrap_or("".to_string()));
                let name = format!(" {}", request.url.clone().unwrap_or("".to_string()));

                ListItem::new(Line::from(vec![
                    if Some(i) == self.selected_index {
                        Span::styled("→ ", Style::default().fg(Color::White))
                    } else {
                        Span::styled("  ", Style::default())
                    },
                    Span::styled(method, method_style),
                    Span::raw(" "),
                    Span::styled(name, name_style),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .style(Style::default().bg(Color::Rgb(20, 20, 20)))
                    .title(Span::styled(
                        if self.focused {
                            " ⦿ Requests "
                        } else {
                            " ○ Requests "
                        },
                        Style::default()
                            .fg(if self.focused {
                                Color::LightBlue
                            } else {
                                Color::Gray
                            })
                            .add_modifier(Modifier::BOLD),
                    )),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Rgb(20, 20, 20)));

        frame.render_widget(list, rect);
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

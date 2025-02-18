use crate::components::Component;
use crate::persistence::{ProjectUpdate, RequestData};
use crate::theme::*;
use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
    Frame,
};

pub enum SidebarAction {
    Noop,
    Selected(RequestData),
    ProjectUpdate(ProjectUpdate),
    EditRequest,
}

pub struct Sidebar {
    rect: Option<Rect>,
    requests: Vec<RequestData>,
    selected_index: Option<usize>,
}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {
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
            KeyCode::Char('e') => Some(SidebarAction::EditRequest),
            KeyCode::Enter => self
                .selected_index
                .map(|i| SidebarAction::Selected(self.requests[i].clone())),
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
                    if let Some(action) = self.handle_selection(key_event.code) {
                        return action;
                    }
                }
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        if let Some(rect) = self.rect {
                            if self.is_mouse_over(mouse_event, &rect) {
                                let relative_y = mouse_event.row.saturating_sub(rect.y + 1); // +1 for border
                                if (relative_y as usize) < self.requests.len() {
                                    self.selected_index = Some(relative_y as usize);
                                    return SidebarAction::Selected(
                                        self.requests[relative_y as usize].clone(),
                                    );
                                }
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

    fn render(&mut self, frame: &mut Frame, rect: Rect, theme: &Theme) {
        self.rect = Some(rect);

        let items: Vec<ListItem> = self
            .requests
            .iter()
            .enumerate()
            .map(|(i, request)| {
                let is_selected = Some(i) == self.selected_index;
                let method_style = if let Some(method) = &request.method {
                    match method.as_str() {
                        "GET" => Style::default().fg(theme.http_methods.get),
                        "POST" => Style::default().fg(theme.http_methods.post),
                        "PUT" => Style::default().fg(theme.http_methods.put),
                        "DELETE" => Style::default().fg(theme.http_methods.delete),
                        "PATCH" => Style::default().fg(theme.http_methods.patch),
                        "HEAD" => Style::default().fg(theme.http_methods.head),
                        _ => Style::default().fg(theme.http_methods.default),
                    }
                } else {
                    Style::default().fg(theme.http_methods.default)
                }
                .add_modifier(Modifier::BOLD)
                .bg(if is_selected {
                    theme.sidebar.selected_bg
                } else {
                    theme.sidebar.bg
                });

                let name_style =
                    Style::default()
                        .fg(theme.sidebar.text_unfocused)
                        .bg(if is_selected {
                            theme.sidebar.selected_bg
                        } else {
                            theme.sidebar.bg
                        });

                let method = format!(" {} ", request.method.clone().unwrap_or("".to_string()));
                let name = format!(" {}", request.name.clone());

                ListItem::new(Line::from(vec![
                    Span::styled(
                        if is_selected { "→ " } else { "  " },
                        Style::default().bg(if is_selected {
                            theme.sidebar.selected_bg
                        } else {
                            theme.sidebar.bg
                        }),
                    ),
                    Span::styled(method, method_style),
                    Span::raw(" "),
                    Span::styled(name, name_style),
                ]))
                .style(Style::default().bg(if is_selected {
                    theme.sidebar.selected_bg
                } else {
                    theme.sidebar.bg
                }))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .style(Style::default().bg(theme.sidebar.bg))
                    .title(Span::styled(
                        " ⦿ Requests ",
                        Style::default()
                            .fg(theme.sidebar.title_focused)
                            .add_modifier(Modifier::BOLD),
                    )),
            )
            .style(Style::default().bg(theme.sidebar.bg));

        frame.render_widget(list, rect);
    }
}

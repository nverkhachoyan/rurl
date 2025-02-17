use crate::actions::Action;
use crate::components::Component;
use crate::persistence::ProjectData;
use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
    Frame,
};
pub enum HeaderAction {
    Noop,
    Focused(bool),
    Render,
    TabChanged(usize),
    CreateProject,
}

pub struct ProjectTab {
    pub id: String,
    pub name: String,
}

pub struct Header {
    focused: bool,
    rect: Option<Rect>,
    projects: Vec<ProjectTab>,
    active_tab: usize,
    current_project: Option<ProjectTab>,
}

impl Header {
    pub fn new(projects: Vec<ProjectTab>, current_project: Option<ProjectTab>) -> Self {
        Header {
            focused: false,
            rect: None,
            projects,
            active_tab: 0,
            current_project,
        }
    }

    pub fn add_project(&mut self, project: ProjectTab) {
        self.projects.push(project);
    }

    pub fn handle_key_event(&mut self, key: KeyCode) -> HeaderAction {
        match key {
            KeyCode::Char('t') => {
                if self.projects.is_empty() {
                    HeaderAction::CreateProject
                } else {
                    HeaderAction::Noop
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if !self.projects.is_empty() && self.active_tab > 0 {
                    self.active_tab -= 1;
                    HeaderAction::TabChanged(self.active_tab)
                } else {
                    HeaderAction::Noop
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if !self.projects.is_empty() && self.active_tab < self.projects.len() - 1 {
                    self.active_tab += 1;
                    HeaderAction::TabChanged(self.active_tab)
                } else {
                    HeaderAction::Noop
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let num = c.to_digit(10).unwrap() as usize;
                if num > 0 && num <= self.projects.len() {
                    self.active_tab = num - 1;
                    HeaderAction::TabChanged(self.active_tab)
                } else {
                    HeaderAction::Noop
                }
            }
            _ => HeaderAction::Noop,
        }
    }
}

impl Component for Header {
    type Action = HeaderAction;

    fn tick(&mut self, event: Option<&Event>, tick_count: u32) -> Self::Action {
        if let Some(event) = event {
            match event {
                Event::Key(key_event) => {
                    if self.focused {
                        return self.handle_key_event(key_event.code);
                    }
                }
                Event::Mouse(mouse_event) => {
                    match mouse_event.kind {
                        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                            if let Some(rect) = self.rect {
                                if self.is_mouse_over(mouse_event, &rect) {
                                    self.focused = true;

                                    // Check if click was on a tab
                                    if !self.projects.is_empty() {
                                        let inner_rect =
                                            Block::default().borders(Borders::ALL).inner(rect);

                                        if mouse_event.row == inner_rect.y {
                                            let tab_width = (inner_rect.width as usize - 1)
                                                / self.projects.len();
                                            let relative_x =
                                                (mouse_event.column - inner_rect.x) as usize;
                                            let clicked_tab = relative_x / tab_width;

                                            if clicked_tab < self.projects.len() {
                                                self.active_tab = clicked_tab;
                                                return HeaderAction::TabChanged(clicked_tab);
                                            }
                                        }
                                    }

                                    return HeaderAction::Focused(true);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        HeaderAction::Noop
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

        frame.render_widget(block.clone(), rect);
        let inner_rect = block.inner(rect);

        if self.projects.is_empty() {
            let help_text = "Press SPACE + t to create a new project";
            let paragraph = Paragraph::new(help_text)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, inner_rect);
        } else {
            let titles: Vec<Line> = self
                .projects
                .iter()
                .enumerate()
                .map(|(i, project)| {
                    let (number_style, name_style) = if i == self.active_tab {
                        (
                            Style::default()
                                .fg(Color::LightYellow)
                                .add_modifier(Modifier::BOLD),
                            Style::default()
                                .fg(Color::LightYellow)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else if self.focused {
                        (
                            Style::default().fg(Color::DarkGray),
                            Style::default().fg(Color::White),
                        )
                    } else {
                        (
                            Style::default().fg(Color::DarkGray),
                            Style::default().fg(Color::Gray),
                        )
                    };

                    let prefix = if i == self.active_tab { "▸" } else { " " };
                    Line::from(vec![
                        Span::styled(format!("{}{}", prefix, i + 1), number_style),
                        Span::raw(" "),
                        Span::styled(
                            if i == self.active_tab {
                                format!("{} ", project.name)
                            } else {
                                format!("{}  ", project.name)
                            },
                            name_style,
                        ),
                    ])
                })
                .collect();

            let tabs = Tabs::new(titles)
                .block(Block::default())
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .fg(Color::LightYellow)
                        .add_modifier(Modifier::BOLD),
                )
                .select(self.active_tab)
                .divider("|");

            frame.render_widget(tabs, inner_rect);
        }
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

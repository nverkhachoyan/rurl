use crate::components::Component;
use crate::theme::Theme;

use crossterm::event::{Event, KeyCode, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Tabs},
    Frame,
};
pub enum HeaderAction {
    Noop,
    TabChanged(usize),
    DeleteProject(usize),
}

pub struct ProjectTab {
    pub name: String,
}

pub struct Header {
    rect: Option<Rect>,
    projects: Vec<ProjectTab>,
    selected_index: usize,
}

impl Header {
    pub fn new(projects: Vec<ProjectTab>) -> Self {
        Header {
            rect: None,
            projects,
            selected_index: 0,
        }
    }

    pub fn add_project(&mut self, project: ProjectTab) {
        self.projects.push(project);
    }

    pub fn handle_key_event(&mut self, key: KeyCode) -> HeaderAction {
        match key {
            KeyCode::Char('d') => {
                if !self.projects.is_empty() {
                    HeaderAction::DeleteProject(self.selected_index)
                } else {
                    HeaderAction::Noop
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if !self.projects.is_empty() && self.selected_index > 0 {
                    self.selected_index -= 1;
                    HeaderAction::TabChanged(self.selected_index)
                } else {
                    HeaderAction::Noop
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if !self.projects.is_empty() && self.selected_index < self.projects.len() - 1 {
                    self.selected_index += 1;
                    HeaderAction::TabChanged(self.selected_index)
                } else {
                    HeaderAction::Noop
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let num = c.to_digit(10).unwrap() as usize;
                if num > 0 && num <= self.projects.len() {
                    self.selected_index = num - 1;
                    HeaderAction::TabChanged(self.selected_index)
                } else {
                    HeaderAction::Noop
                }
            }
            _ => HeaderAction::Noop,
        }
    }

    fn calculate_tab_positions(&self) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        let mut current_x = 0;

        for (_, project) in self.projects.iter().enumerate() {
            let tab_width = 3 + project.name.len() + 1;
            positions.push((current_x, current_x + tab_width));
            current_x += tab_width;
        }

        positions
    }

    fn handle_mouse_event(&mut self, mouse_event: &MouseEvent, rect: &Rect) -> HeaderAction {
        let relative_x = mouse_event.column.saturating_sub(rect.x);
        let tab_positions = self.calculate_tab_positions();

        for (index, (start, end)) in tab_positions.iter().enumerate() {
            if relative_x >= *start as u16 && relative_x < *end as u16 {
                self.selected_index = index;
                return HeaderAction::TabChanged(index);
            }
        }

        HeaderAction::Noop
    }
}

impl Component for Header {
    type Action = HeaderAction;

    fn tick(&mut self, event: Option<&Event>, _: u32) -> Self::Action {
        if let Some(event) = event {
            match event {
                Event::Key(key_event) => {
                    return self.handle_key_event(key_event.code);
                }
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        if let Some(rect) = self.rect {
                            if self.is_mouse_over(mouse_event, &rect) {
                                return self.handle_mouse_event(mouse_event, &rect);
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        HeaderAction::Noop
    }

    fn render(&mut self, frame: &mut Frame, rect: Rect, theme: &Theme) {
        self.rect = Some(rect);

        let block = Block::default().style(Style::default().bg(theme.general.content_bg));

        frame.render_widget(block.clone(), rect);
        let inner_rect = block.inner(rect);

        if self.projects.is_empty() {
            let help_text = "Press SPACE + t to create a new project";
            let paragraph = Paragraph::new(Line::from(vec![
                Span::styled(
                    " HELP ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(theme.general.title_focused)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(help_text, Style::default().fg(theme.general.text_unfocused)),
            ]))
            .style(Style::default().bg(theme.general.content_bg))
            .alignment(Alignment::Center);
            frame.render_widget(paragraph, inner_rect);
        } else {
            let titles: Vec<Line> = self
                .projects
                .iter()
                .enumerate()
                .map(|(i, project)| {
                    let is_active = i == self.selected_index;
                    let (number_style, name_style) = if is_active {
                        (
                            Style::default()
                                .fg(Color::Black)
                                .bg(theme.general.title_focused)
                                .add_modifier(Modifier::BOLD),
                            Style::default()
                                .fg(Color::Black)
                                .bg(theme.general.title_focused)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        (
                            Style::default().fg(theme.general.text_unfocused),
                            Style::default().fg(theme.general.text_unfocused),
                        )
                    };

                    Line::from(vec![
                        Span::styled(format!(" {} ", i + 1), number_style),
                        Span::styled(format!("{} ", project.name), name_style),
                    ])
                })
                .collect();

            let tabs = Tabs::new(titles)
                .block(Block::default())
                .style(Style::default().bg(theme.general.content_bg))
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(theme.general.title_focused)
                        .add_modifier(Modifier::BOLD),
                )
                .select(self.selected_index)
                .divider("|");

            frame.render_widget(tabs, inner_rect);
        }
    }
}

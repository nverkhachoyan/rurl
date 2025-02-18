use crossterm::event::Event;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{components::Component, theme::Theme};

pub enum FooterAction {
    Noop,
}

pub struct Footer {
    status: String,
    rect: Option<Rect>,
    mode: String,
}

impl Footer {
    pub fn new() -> Self {
        Footer {
            status: String::from("Ready"),
            rect: None,
            mode: String::from("NORMAL"),
        }
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn set_mode(&mut self, mode: String) {
        self.mode = mode;
    }

    fn render_mode_indicator(&self, mode: &str, color: Color, theme: &Theme) -> Vec<Span<'static>> {
        vec![Span::styled(
            format!(" {} ", mode),
            Style::default()
                .fg(theme.footer.key_bg)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        )]
    }

    fn render_command(
        &self,
        key: &str,
        desc: &str,
        color: Color,
        theme: &Theme,
    ) -> Vec<Span<'static>> {
        vec![
            Span::raw(" ".to_string()),
            Span::styled(
                format!(" {} ", key),
                Style::default()
                    .fg(theme.footer.key_bg)
                    .bg(color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ".to_string()),
            Span::styled(
                desc.to_string(),
                Style::default().fg(theme.footer.description),
            ),
        ]
    }

    pub fn render_status(&self, mode: &str, theme: &Theme) -> Line<'static> {
        match mode {
            "NORMAL" => {
                let mut spans =
                    self.render_mode_indicator("NORMAL", theme.footer.mode_normal, theme);
                spans.extend(self.render_command(
                    "SPACE",
                    "command mode",
                    theme.footer.mode_command,
                    theme,
                ));
                Line::from(spans)
            }
            "COMMAND" => {
                let mut spans =
                    self.render_mode_indicator("COMMAND", theme.footer.mode_command, theme);
                spans.extend(self.render_command("t", "tab mode", theme.footer.mode_tab, theme));
                spans.extend(self.render_command(
                    "c",
                    "create project",
                    theme.footer.mode_create,
                    theme,
                ));
                spans.extend(self.render_command("q", "quit", theme.http_methods.delete, theme));
                spans.extend(self.render_command(
                    "n",
                    "normal mode",
                    theme.footer.mode_normal,
                    theme,
                ));
                spans.extend(self.render_command(
                    "SPACE",
                    "toggle mode",
                    theme.footer.mode_command,
                    theme,
                ));
                Line::from(spans)
            }
            "TAB" => {
                let mut spans = self.render_mode_indicator("TAB", theme.footer.mode_tab, theme);
                spans.extend(self.render_command(
                    "h/l",
                    "switch tabs",
                    theme.http_methods.get,
                    theme,
                ));
                spans.extend(self.render_command(
                    "1-9",
                    "select tab",
                    theme.footer.mode_command,
                    theme,
                ));
                spans.extend(self.render_command(
                    "d",
                    "delete project",
                    theme.http_methods.delete,
                    theme,
                ));
                spans.extend(self.render_command("ESC", "back", theme.http_methods.delete, theme));
                Line::from(spans)
            }
            "CREATE" => {
                let mut spans =
                    self.render_mode_indicator("CREATE", theme.footer.mode_create, theme);
                spans.extend(self.render_command(
                    "ENTER",
                    "confirm",
                    theme.footer.mode_normal,
                    theme,
                ));
                spans.extend(self.render_command(
                    "ESC",
                    "cancel",
                    theme.http_methods.delete,
                    theme,
                ));
                Line::from(spans)
            }
            _ => Line::from(vec![Span::raw(self.status.clone())]),
        }
    }
}

impl Component for Footer {
    type Action = FooterAction;

    fn tick(&mut self, _: Option<&Event>, _: u32) -> Self::Action {
        FooterAction::Noop
    }

    fn render(&mut self, frame: &mut Frame, rect: Rect, theme: &Theme) {
        self.rect = Some(rect);

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.footer.border))
            .style(Style::default().bg(theme.footer.bg));

        let inner_rect = block.inner(rect);
        frame.render_widget(block, rect);

        let status = Paragraph::new(self.render_status(&self.mode, theme))
            .alignment(Alignment::Left)
            .style(Style::default().bg(theme.footer.bg));

        frame.render_widget(status, inner_rect);
    }
}

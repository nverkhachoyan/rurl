use ratatui::{
    layout::{Constraint, Direction, Layout as TuiLayout, Rect},
    Frame,
};

use crate::{components::Component, theme::Theme};

pub enum LayoutAction {
    Noop,
}

pub struct AppLayout {
    pub header_height: u16,
    pub footer_height: u16,
    pub sidebar_width: u16,
}

impl AppLayout {
    pub fn new() -> Self {
        Self {
            header_height: 1,
            footer_height: 3,
            sidebar_width: 30,
        }
    }

    pub fn get_layout_areas(&self, frame_area: Rect) -> (Rect, Rect, Rect, Rect) {
        let vertical_chunks = TuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(self.header_height),
                Constraint::Min(0),
                Constraint::Length(self.footer_height),
            ])
            .split(frame_area);

        let middle_chunks = TuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(self.sidebar_width), Constraint::Min(0)])
            .split(vertical_chunks[1]);

        (
            vertical_chunks[0], // header
            middle_chunks[0],   // sidebar
            middle_chunks[1],   // main content
            vertical_chunks[2], // footer
        )
    }
}

impl Component for AppLayout {
    type Action = LayoutAction;

    fn tick(&mut self, _: Option<&crossterm::event::Event>, _: u32) -> LayoutAction {
        LayoutAction::Noop
    }

    fn render(&mut self, _: &mut Frame, _: Rect, _: &Theme) {}
}

use crossterm::event::MouseEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout as TuiLayout, Rect},
    Frame,
};

use crate::actions::Action;
use crate::components::Component;

pub enum LayoutAction {
    Noop,
    Render,
}

pub struct Layout {
    header_height: u16,
    footer_height: u16,
    sidebar_width: u16,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            header_height: 3,
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

impl Component for Layout {
    type Action = LayoutAction;

    fn tick(&mut self, _: Option<&crossterm::event::Event>, _: u32) -> LayoutAction {
        LayoutAction::Noop
    }

    fn render(&mut self, _: &mut Frame, rect: Rect) {
        let _ = self.get_layout_areas(rect);
        // Implemented in the App struct
    }

    fn focus(&mut self, _: bool) {}
}

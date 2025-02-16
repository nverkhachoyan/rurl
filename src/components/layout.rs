use ratatui::{
    layout::{Constraint, Direction, Layout as TuiLayout, Rect},
    Frame,
};

use crate::actions::Action;
use crate::components::Component;

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
    fn handle_event(&mut self, _event: Option<&crossterm::event::Event>) -> Action {
        Action::Noop
    }

    fn render(&mut self, _: &mut Frame, rect: Rect) {
        let _ = self.get_layout_areas(rect);
        // The actual rendering will be done by the App, which will use these areas
        // to position other components
    }
}

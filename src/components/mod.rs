use crossterm::event::{Event, MouseEvent};
use ratatui::{layout::Rect, Frame};

use crate::theme::Theme;

mod content;
mod footer;
mod header;
mod layout;
mod sidebar;

pub use content::{Content, ContentAction};
pub use footer::Footer;
pub use header::{Header, HeaderAction, ProjectTab};
pub use layout::AppLayout;
pub use sidebar::{Sidebar, SidebarAction};

pub trait Component {
    type Action;

    fn tick(&mut self, event: Option<&Event>, tick_count: u32) -> Self::Action;
    fn render(&mut self, frame: &mut Frame, rect: Rect, theme: &Theme);
    fn is_mouse_over(&self, mouse_event: &MouseEvent, rect: &Rect) -> bool {
        let (x, y) = (mouse_event.column, mouse_event.row);
        x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
    }
}

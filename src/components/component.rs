use crate::actions::Action;
use ratatui::{
    crossterm::event::{Event, KeyEvent, MouseEvent},
    layout::Rect,
    Frame,
};

pub trait Component {
    fn handle_event(&mut self, event: Option<&Event>) -> Action {
        match event {
            Some(Event::Key(key_event)) => self.handle_key_event(*key_event),
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(*mouse_event),
            // Some(Event::Resize(x, y)) => Action::Resize(*x, *y),
            Some(_) => Action::Noop,
            None => Action::Noop,
        }
    }
    fn handle_key_event(&mut self, _: KeyEvent) -> Action {
        Action::Noop
    }
    fn handle_mouse_event(&mut self, _: MouseEvent) -> Action {
        Action::Noop
    }
    fn render(&mut self, f: &mut Frame, rect: Rect);
    fn focus(&mut self, _: bool) {}
}

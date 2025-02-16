use crate::actions::Action;
use crate::components::Component;
use crossterm::event::{Event, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub struct Header {
    id: u16,
    title: String,
    subtitle: Option<String>,
    focused: bool,
}

impl Header {
    pub fn new(title: &str) -> Self {
        Header {
            id: 0,
            title: title.to_string(),
            subtitle: None,
            focused: false,
        }
    }
}

impl Component for Header {
    fn handle_event(&mut self, _event: Option<&Event>) -> Action {
        match _event {
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(*mouse_event),
            _ => Action::Noop,
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> Action {
        match event.kind {
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                self.focus(true);
                Action::Render
            }
            _ => Action::Noop,
        }
    }

    fn render(&mut self, frame: &mut Frame, _rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if self.focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            });

        let color = if self.focused {
            Color::Yellow
        } else {
            Color::White
        };
        let title = Paragraph::new(self.title.clone())
            .alignment(Alignment::Center)
            .style(Style::default().fg(color));

        frame.render_widget(block, frame.area());
        frame.render_widget(title, frame.area());
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::actions::Action;
use crate::components::{Component, Header, Layout};

pub struct App {
    layout: Layout,
    header: Header,
    components: Vec<Box<dyn Component>>,
}

impl App {
    pub fn new() -> Self {
        let layout = Layout::new();
        let header = Header::new("Welcome to Rurl!");
        App {
            layout,
            header,
            components: Vec::new(),
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> Action {
        if let Event::Key(key) = event {
            if let KeyCode::Char('q') = key.code {
                return Action::Quit;
            }
        }

        // Handle header events
        let action = self.header.handle_event(Some(event));
        match action {
            Action::Render | Action::Quit => return action,
            _ => {}
        }

        // Handle other component events
        for component in self.components.iter_mut() {
            let action = component.handle_event(Some(event));
            match action {
                Action::Render | Action::Quit => return action,
                _ => {}
            }
        }
        Action::Noop
    }

    pub fn render(&mut self, frame: &mut Frame) {
        // Get the layout areas
        let (header_area, sidebar_area, main_area, footer_area) =
            self.layout.get_layout_areas(frame.area());

        // Render the header
        self.header.render(frame, header_area);

        // Render the sidebar
        let sidebar = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Sidebar");
        frame.render_widget(sidebar, sidebar_area);

        // Render the main content area
        let main_content = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Main Content");
        frame.render_widget(main_content, main_area);

        // Render the footer
        let footer = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let footer_text = Paragraph::new("Press 'q' to quit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(footer, footer_area);
        frame.render_widget(footer_text, footer_area);

        // Render other components
        for component in self.components.iter_mut() {
            component.render(frame, frame.area());
        }
    }
}

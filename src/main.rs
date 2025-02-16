mod actions;
mod components;

use actions::Action;
use crossterm::event::{self};
use std::error::Error;

mod app;
mod tui;
use app::App;
use tui::Tui;

fn main() -> Result<(), Box<dyn Error>> {
    let mut tui = Tui::new()?;
    let mut app = App::new();

    loop {
        tui.terminal.draw(|f| app.render(f))?;
        let event = event::read()?;
        let action = app.handle_event(&event);
        let mut frame = tui.terminal.get_frame();
        match action {
            Action::Quit => break,
            Action::Render => app.render(&mut frame),
            _ => {}
        }
    }

    tui.destroy()?;
    Ok(())
}

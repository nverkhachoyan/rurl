use crossterm::event;
use std::{error::Error, time::Duration};

mod actions;
mod app;
mod components;
mod persistence;
mod tui;

use app::{App, AppAction};
use persistence::Storage;
use tui::Tui;

fn main() -> Result<(), Box<dyn Error>> {
    let mut tui = Tui::new()?;
    let storage = Storage::new();
    let mut app = App::new(storage);

    loop {
        if app.should_render() {
            tui.terminal.draw(|f| app.render(f))?;
        }

        if event::poll(Duration::from_millis(250))? {
            let event = event::read()?;
            let action = app.tick(Some(&event));
            if let AppAction::Quit = action {
                break;
            }
        } else {
            app.tick(None);
        }
    }

    tui.destroy()?;
    Ok(())
}

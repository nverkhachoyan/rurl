use crossterm::event;
use std::{error::Error, time::Duration};

mod app;
mod components;
mod config;
mod persistence;
mod theme;
mod tui;

use app::{App, AppAction};
use config::Config;
use persistence::Storage;
use tui::Tui;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::load();
    let theme = config.create_theme();

    let mut tui = Tui::new()?;
    let storage = Storage::new();
    let mut app = App::new(storage, theme);

    tui.terminal.draw(|f| app.render(f))?;

    loop {
        if app.should_render() {
            tui.terminal.draw(|f| app.render(f))?;
        }

        if event::poll(Duration::from_millis(50))? {
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

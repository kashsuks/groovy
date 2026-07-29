mod app;
mod tui;

use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use app::App;

fn main() -> color_eyre::Result<()> {
    install_panic_hook()?;

    let mut terminal = tui::init()?;
    let result = run(&mut terminal);
    tui::restore()?;

    result
}

fn run(terminal: &mut tui::Tui) -> color_eyre::Result<()> {
    let mut app = App::new();

    while !app.should_quit {
        terminal.draw(|frame| tui::ui::draw(frame, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // on some terminals a single physical keypress produces both
                // a Press and a Release event; only act on Press to avoid
                // double-handling
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }
    }

    Ok(())
}

// make sure a panic still leaves the terminal in a useable state
// a panic mid-render leaves the shell stuck in raw mode
fn install_panic_hook() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();
    let panic_hook = panic_hook.into_panic_hook();
    let eyre_hook = eyre_hook.into_eyre_hook();

    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = tui::restore();
        eyre_hook(e)
    }))?;

    std::panic::set_hook(Box::new(move |info| {
        let _ = tui::restore();
        panic_hook(info);
    }));

    Ok(())
}

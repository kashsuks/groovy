mod app;
mod config;
mod library;
mod player;
mod tui;

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};

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
    let mut last_screen = app.screen;

    while !app.should_quit {
        app.poll_player();

        // ratatui only redraws cells that changed since the previous frame;
        // widgets that render fewer rows than their area (e.g. a short track
        // table) can leave stale glyphs from the previous screen behind.
        // Forcing a full clear on screen transitions guarantees a clean slate.
        if app.screen != last_screen {
            terminal.clear()?;
            last_screen = app.screen;
        }

        terminal.draw(|frame| tui::ui::draw(frame, &app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // on some terminals a single physical keypress produces both
                    // a Press and a Release event; only act on Press to avoid
                    // double-handling
                    if key.kind == KeyEventKind::Press {
                        // Ctrl+C quits from anywhere, including text-entry modes
                        // (naming a playlist, typing a path) where 'q' is just a
                        // regular character and can't double as a quit key.
                        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            app.should_quit = true;
                        } else {
                            app.handle_key(key.code);
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                        handle_bottom_bar_click(&mut app, terminal.size()?, mouse.column, mouse.row);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Maps a left-click at (col, row) onto the transport buttons in the
/// persistent bottom bar, if it landed on one. Recomputes the exact same
/// geometry `tui::ui::draw` used, rather than tracking rendered rects, since
/// the layout is pure geometry (doesn't depend on app state).
fn handle_bottom_bar_click(app: &mut App, terminal_size: ratatui::layout::Rect, col: u16, row: u16) {
    let (_, bottom_bar_area) = tui::ui::split_content_and_bar(terminal_size);
    let layout = tui::ui::bottom_bar_layout(bottom_bar_area);

    let hit = |rect: ratatui::layout::Rect| {
        col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
    };

    let button = if hit(layout.prev) {
        Some(app::ControlButton::Prev)
    } else if hit(layout.play_pause) {
        Some(app::ControlButton::PlayPause)
    } else if hit(layout.next) {
        Some(app::ControlButton::Next)
    } else if hit(layout.shuffle) {
        Some(app::ControlButton::Shuffle)
    } else if hit(layout.repeat) {
        Some(app::ControlButton::Repeat)
    } else {
        None
    };

    if let Some(button) = button {
        button.activate(app);
    }
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

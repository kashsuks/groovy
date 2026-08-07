use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::{HomeMode, browser::BrowserState};
use crate::app::{App, PlaybackState, Screen};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.size();

    let screen_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(4)])
        .split(area);
    let content_area = screen_chunks[0];
    let bottom_bar_area = screen_chunks[1];

    match app.screen {
        Screen::Home => draw_home(frame, app, content_area),
        Screen::Playlist => draw_playlist(frame, app, content_area),
        Screen::Cinema => draw_cinema(frame, app, content_area),
    }

    draw_bottom_bar(frame, app, bottom_bar_area);

    if app.sidebar_open {
        draw_sidebar(frame, app);
    }
}

fn draw_home(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let logo = Paragraph::new(vec![
        Line::from(Span::styled(
            "Groovy",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("your playlists, from disk"),
    ])
    .alignment(Alignment::Center)
    .block(Block::default());

    frame.render_widget(logo, chunks[0]);

    draw_playlist_list(frame, app, chunks[1]);

    if app.home_mode == HomeMode::Browsing {
        if let Some(browser) = &app.browser {
            draw_browser(frame, browser, area);

            if let Some(popup) = &browser.popup {
                draw_path_popup(frame, &popup.input, area);
            }
        }
    }

    if app.home_mode == HomeMode::Naming {
        if let Some(browser) = &app.browser {
            draw_browser(frame, browser, area);
        }
        draw_naming_prompt(frame, &app.naming_input, area);
    }
}

fn draw_naming_prompt(frame: &mut Frame, input: &str, area: Rect) {
    let popup_area = centered_rect(60, 3, area);

    let popup = Paragraph::new(format!("{input}_"))
        .block(Block::default().borders(Borders::ALL).title(" Playlist name "));

    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

fn draw_path_popup(frame: &mut Frame, input: &str, area: Rect) {
    let popup_area = centered_rect(60, 3, area);

    let popup = Paragraph::new(format!("{input}_"))
        .block(Block::default().borders(Borders::ALL).title(" Go to path "));

    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

fn centered_rect(percent_width: u16, height: u16, area: Rect) -> Rect {
    let width = area.width * percent_width / 100;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect { x, y, width, height }
}

fn draw_browser(frame: &mut Frame, browser: &BrowserState, area: Rect) {
    // The list only paints cells for its border and its actual entries; without
    // an explicit Clear first, whatever the Home screen drew underneath this
    // frame (logo text, playlist rows) shows through the gaps.
    frame.render_widget(ratatui::widgets::Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let items: Vec<ListItem> = browser
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let base_style = if entry.is_dir {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let style = if i == browser.selected {
                base_style.bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                base_style
            };
            ListItem::new(format!(" {}", entry.name)).style(style)
        })
        .collect();

    let title = format!(" {}", browser.current_dir.display());
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(list, chunks[0]);

    // Browsing here only picks a folder — it doesn't create a playlist by
    // itself, which is easy to miss without this hint.
    let hint = Paragraph::new(
        "[Enter/l] open dir   [h/Backspace] up   [/] type path   [s] save this folder as a playlist   [Esc] cancel",
    )
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, chunks[1]);
}

fn draw_playlist_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.config.playlists.is_empty() {
        let placeholder = Paragraph::new("No playlists yet - press 'n' to create one")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Playlists "));
        frame.render_widget(placeholder, area);
        return;
    }

    let items: Vec<ListItem> = app
        .config
        .playlists
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == app.home_selected {
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} ({})", entry.name, entry.path.display())).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Playlists "));

    frame.render_widget(list, area);
}

fn draw_playlist(frame: &mut Frame, app: &App, area: Rect) {
    let Some(playlist) = &app.current_playlist else {
        let empty = Paragraph::new("No playlist open")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Playlist "));
        frame.render_widget(empty, area);
        return;
    };

    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    let title = Paragraph::new(Line::from(Span::styled(
        playlist.name.as_str(),
        Style::default().add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Left)
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, body_chunks[0]);

    let play_button = Paragraph::new(Span::styled(
        " Play ",
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(play_button, body_chunks[1]);

    draw_track_table(frame, app, playlist, body_chunks[2]);
}

fn draw_track_table(frame: &mut Frame, app: &App, playlist: &crate::app::ActivePlaylist, area: Rect) {
    let header = Row::new(vec!["#", "Title", "Duration"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = playlist
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let is_now_playing = app.now_playing_index == Some(i);
            let is_selected = app.track_selected == i;

            let mut style = Style::default();
            if is_selected {
                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
            }
            if is_now_playing {
                style = style.fg(Color::Green);
            }

            let marker = if is_now_playing { ">" } else { " " };

            Row::new(vec![
                format!("{marker}{}", track.index),
                track.title.clone(),
                format_duration(track.duration),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Min(10),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths).header(header);

    frame.render_widget(table, area);
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn draw_cinema(frame: &mut Frame, _app: &App, area: Rect) {
    let body = Paragraph::new("Cinema mode placeholder")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" Cinema "));

    frame.render_widget(body, area);
}

/// Persistent, full-width transport bar mounted at the bottom of every
/// screen (Home included), not just Playlist/Cinema.
fn draw_bottom_bar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL);

    let playlist = app.current_playlist.as_ref();
    let track = playlist.and_then(|p| app.now_playing_index.and_then(|i| p.tracks.get(i)));

    let Some(track) = track else {
        let message = if playlist.is_some() {
            "nothing playing — select a track and press Enter"
        } else {
            "no playlist open"
        };
        let bar = Paragraph::new(vec![Line::from(message), Line::from(controls_line(app))])
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(bar, area);
        return;
    };

    let state_label = match app.playback_state {
        PlaybackState::Playing => "\u{25b6}",
        PlaybackState::Paused => "\u{23f8}",
        PlaybackState::Stopped => "\u{25a0}",
    };

    let elapsed = format_duration(app.position);
    let total = format_duration(track.duration);

    // Stretch the progress bar to fill the available width instead of a
    // fixed size, since this panel now spans the whole terminal.
    let bar_width = (area.width as usize).saturating_sub(24).clamp(10, 80);
    let progress_ratio = if track.duration.as_secs_f64() > 0.0 {
        (app.position.as_secs_f64() / track.duration.as_secs_f64()).min(1.0)
    } else {
        0.0
    };
    let filled = (bar_width as f64 * progress_ratio).round() as usize;
    let progress_bar = format!(
        "{}{}",
        "=".repeat(filled),
        "-".repeat(bar_width.saturating_sub(filled))
    );

    let now_playing_line = format!("{state_label}  {}   {elapsed} [{progress_bar}] {total}", track.title);

    let bar = Paragraph::new(vec![
        Line::from(now_playing_line),
        Line::from(controls_line(app)),
    ])
    .alignment(Alignment::Center)
    .block(block.title(" Now Playing "));
    frame.render_widget(bar, area);
}

fn controls_line(app: &App) -> String {
    format!(
        "[p] Prev   [Space] Play/Pause   [n] Next   [s] Shuffle: {}   [r] Repeat: {}",
        if app.shuffle { "On" } else { "Off" },
        app.repeat_mode.label(),
    )
}

/// Sidebar overlay: a floating panel over whatever is active on the screen
fn draw_sidebar(frame: &mut Frame, _app: &App) {
    let area = frame.size();
    let width = (area.width / 4).max(20);
    let sidebar_area = Rect {
        x: 0,
        y: 0,
        width,
        height: area.height,
    };

    let sidebar = Paragraph::new("Sidebar placeholder")
        .block(Block::default().borders(Borders::ALL).title(" Playlists "));

    frame.render_widget(ratatui::widgets::Clear, sidebar_area);
    frame.render_widget(sidebar, sidebar_area);
}

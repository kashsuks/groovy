use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::{HomeMode, browser::BrowserState};
use crate::app::{App, PlaybackState, Screen};
use crate::tui::theme;

/// Splits the terminal into the screen-specific content area and the
/// persistent bottom bar area. Shared with the mouse click handler in
/// `main.rs` so hit-testing lines up with what was actually drawn.
/// Bordered block styled with the Catppuccin Macchiato palette, used for
/// every titled panel so borders/titles stay consistent across screens.
fn themed_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::SURFACE2))
        .title(Span::styled(title, Style::default().fg(theme::LAVENDER)))
}

pub fn split_content_and_bar(area: Rect) -> (Rect, Rect) {
    let screen_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(4)])
        .split(area);
    (screen_chunks[0], screen_chunks[1])
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.size();
    let (content_area, bottom_bar_area) = split_content_and_bar(area);

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
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "your playlists, from disk",
            Style::default().fg(theme::SUBTEXT0),
        )),
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
        .style(Style::default().fg(theme::TEXT).bg(theme::MANTLE))
        .block(themed_block(" Playlist name "));

    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

fn draw_path_popup(frame: &mut Frame, input: &str, area: Rect) {
    let popup_area = centered_rect(60, 3, area);

    let popup = Paragraph::new(format!("{input}_"))
        .style(Style::default().fg(theme::TEXT).bg(theme::MANTLE))
        .block(themed_block(" Go to path "));

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
                Style::default().fg(theme::SAPPHIRE)
            } else {
                Style::default().fg(theme::SUBTEXT0)
            };
            let style = if i == browser.selected {
                base_style.bg(theme::SURFACE1).add_modifier(Modifier::BOLD)
            } else {
                base_style
            };
            ListItem::new(format!(" {}", entry.name)).style(style)
        })
        .collect();

    let title = format!(" {}", browser.current_dir.display());
    let list = List::new(items).block(themed_block(&title));
    frame.render_widget(list, chunks[0]);

    // Browsing here only picks a folder — it doesn't create a playlist by
    // itself, which is easy to miss without this hint.
    let hint = Paragraph::new(
        "[Enter/l] open dir   [h/Backspace] up   [/] type path   [s] save this folder as a playlist   [Esc] cancel",
    )
    .alignment(Alignment::Center)
    .style(Style::default().fg(theme::OVERLAY1));
    frame.render_widget(hint, chunks[1]);
}

fn draw_playlist_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.config.playlists.is_empty() {
        let placeholder = Paragraph::new("No playlists yet - press 'n' to create one")
            .style(Style::default().fg(theme::SUBTEXT0))
            .alignment(Alignment::Center)
            .block(themed_block(" Playlists "));
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
                Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT)
            };
            ListItem::new(format!("{} ({})", entry.name, entry.path.display())).style(style)
        })
        .collect();

    let list = List::new(items).block(themed_block(" Playlists "));

    frame.render_widget(list, area);
}

fn draw_playlist(frame: &mut Frame, app: &App, area: Rect) {
    let Some(playlist) = &app.current_playlist else {
        let empty = Paragraph::new("No playlist open")
            .style(Style::default().fg(theme::SUBTEXT0))
            .alignment(Alignment::Center)
            .block(themed_block(" Playlist "));
        frame.render_widget(empty, area);
        return;
    };

    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    let title = Paragraph::new(Line::from(Span::styled(
        playlist.name.as_str(),
        Style::default().fg(theme::LAVENDER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Left)
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(theme::SURFACE2)));
    frame.render_widget(title, body_chunks[0]);

    let play_button = Paragraph::new(Span::styled(
        " Play ",
        Style::default()
            .fg(theme::CRUST)
            .bg(theme::GREEN)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(play_button, body_chunks[1]);

    draw_track_table(frame, app, playlist, body_chunks[2]);
}

fn draw_track_table(frame: &mut Frame, app: &App, playlist: &crate::app::ActivePlaylist, area: Rect) {
    let header = Row::new(vec!["#", "Title", "Duration"])
        .style(Style::default().fg(theme::SUBTEXT1).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = playlist
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let is_now_playing = app.now_playing_index == Some(i);
            let is_selected = app.track_selected == i;

            let mut style = Style::default().fg(theme::TEXT);
            if is_selected {
                style = style.bg(theme::SURFACE1).add_modifier(Modifier::BOLD);
            }
            if is_now_playing {
                style = style.fg(theme::GREEN);
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

    let mut state = TableState::default().with_selected(Some(app.track_selected));
    frame.render_stateful_widget(table, area, &mut state);
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
        .style(Style::default().fg(theme::SUBTEXT0))
        .alignment(Alignment::Center)
        .block(themed_block(" Cinema "));

    frame.render_widget(body, area);
}

const BUTTON_WIDTH: u16 = 7;
const BUTTON_GAP: u16 = 2;
const BUTTON_COUNT: u16 = 5;

/// Click/render regions for the five transport icon buttons. Pure geometry —
/// doesn't depend on app state — so `main.rs` can recompute the exact same
/// rects at click time without needing to touch rendering code.
pub struct BottomBarLayout {
    pub prev: Rect,
    pub play_pause: Rect,
    pub next: Rect,
    pub shuffle: Rect,
    pub repeat: Rect,
}

pub fn bottom_bar_layout(area: Rect) -> BottomBarLayout {
    let total_width = BUTTON_COUNT * BUTTON_WIDTH + (BUTTON_COUNT - 1) * BUTTON_GAP;
    let start_x = area.x + area.width.saturating_sub(total_width) / 2;
    let y = area.y + 1; // first content row, just under the top border

    let rect_at = |slot: u16| Rect {
        x: start_x + slot * (BUTTON_WIDTH + BUTTON_GAP),
        y,
        width: BUTTON_WIDTH,
        height: 1,
    };

    BottomBarLayout {
        prev: rect_at(0),
        play_pause: rect_at(1),
        next: rect_at(2),
        shuffle: rect_at(3),
        repeat: rect_at(4),
    }
}

/// Persistent, full-width transport bar mounted at the bottom of every
/// screen (Home included), not just Playlist/Cinema. Icon buttons are both
/// keyboard-driven (see `App::handle_playback_screen_key`) and mouse-clickable
/// (see the click handler in `main.rs`, which reuses `bottom_bar_layout`).
fn draw_bottom_bar(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(themed_block(" Now Playing "), area);

    let layout = bottom_bar_layout(area);

    let inactive = Style::default().fg(theme::SUBTEXT0);
    let active = Style::default().fg(theme::CRUST).bg(theme::GREEN).add_modifier(Modifier::BOLD);
    let repeat_one = Style::default().fg(theme::CRUST).bg(theme::YELLOW).add_modifier(Modifier::BOLD);

    render_icon_button(frame, layout.prev, "\u{23ee}", inactive);
    let (play_icon, play_style) = match app.playback_state {
        PlaybackState::Playing => ("\u{23f8}", active),
        _ => ("\u{25b6}", inactive),
    };
    render_icon_button(frame, layout.play_pause, play_icon, play_style);
    render_icon_button(frame, layout.next, "\u{23ed}", inactive);
    render_icon_button(frame, layout.shuffle, "\u{21c4}", if app.shuffle { active } else { inactive });

    let repeat_icon = if app.repeat_mode == crate::app::RepeatMode::One { "\u{21bb}1" } else { "\u{21bb}" };
    let repeat_style = match app.repeat_mode {
        crate::app::RepeatMode::Off => inactive,
        crate::app::RepeatMode::All => active,
        crate::app::RepeatMode::One => repeat_one,
    };
    render_icon_button(frame, layout.repeat, repeat_icon, repeat_style);

    let playlist = app.current_playlist.as_ref();
    let track = playlist.and_then(|p| app.now_playing_index.and_then(|i| p.tracks.get(i)));

    let progress_text = match track {
        Some(track) => {
            let elapsed = format_duration(app.position);
            let total = format_duration(track.duration);

            // Stretch the progress bar to fill the available width instead of
            // a fixed size, since this panel now spans the whole terminal.
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
            format!("{}   {elapsed} [{progress_bar}] {total}", track.title)
        }
        None if playlist.is_some() => "select a track and press Enter".to_string(),
        None => "no playlist open".to_string(),
    };

    let progress_area = Rect {
        x: area.x + 1,
        y: area.y + 2,
        width: area.width.saturating_sub(2),
        height: 1,
    };
    let progress_line = Paragraph::new(progress_text)
        .style(Style::default().fg(theme::TEXT))
        .alignment(Alignment::Center);
    frame.render_widget(progress_line, progress_area);
}

fn render_icon_button(frame: &mut Frame, rect: Rect, icon: &str, style: Style) {
    let button = Paragraph::new(format!("[{icon}]")).alignment(Alignment::Center).style(style);
    frame.render_widget(button, rect);
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
        .style(Style::default().fg(theme::SUBTEXT0).bg(theme::MANTLE))
        .block(themed_block(" Playlists "));

    frame.render_widget(ratatui::widgets::Clear, sidebar_area);
    frame.render_widget(sidebar, sidebar_area);
}

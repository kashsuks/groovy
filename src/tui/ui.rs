use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::{HomeMode, browser::BrowserState};
use crate::app::{App, Screen};

pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Home => draw_home(frame, app),
        Screen::Playlist => draw_playlist(frame, app),
        Screen::Cinema => draw_cinema(frame, app),
    }

    if app.sidebar_open {
        draw_sidebar(frame, app);
    }
}

fn draw_home(frame: &mut Frame, app: &App) {
    let area = frame.size();

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

    frame.render_widget(list, area);
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

fn draw_playlist(frame: &mut Frame, app: &App) {
    let area = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);
    
    let Some(playlist) = &app.current_playlist else {
        let empty = Paragraph::new("No playlist open")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Playlist "));
        frame.render_widget(empty, chunks[0]);
        draw_bottom_bar(frame, chunks[1]);
        return;
    };

    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1), Constraint::Min(1)])
        .split(chunks[0]);

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

    draw_track_table(frame, playlist, body_chunks[2]);

    draw_bottom_bar(frame, chunks[1]);
}

fn draw_track_table(frame: &mut Frame, playlist: &crate::app::ActivePlaylist, area: Rect) {
    let header = Row::new(vec!["#", "Title", "Duration"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = playlist
        .tracks
        .iter()
        .map(|track| {
            Row::new(vec![
                track.index.to_string(),
                track.title.clone(),
                format_duration(track.duration),
            ])
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

fn draw_cinema(frame: &mut Frame, _app: &App) {
    let area = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    let body = Paragraph::new("Cinema mode placeholder")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" Cinema "));

    frame.render_widget(body, chunks[0]);

    draw_bottom_bar(frame, chunks[1]);
}

// Shared playback control bar, mounted on playlist and cinema screens
fn draw_bottom_bar(frame: &mut Frame, area: Rect) {
    let bar = Paragraph::new("[ now playing: -- ] (bottom bar placeholder)")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(bar, area);
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

    frame.render_widget(sidebar, sidebar_area);
}

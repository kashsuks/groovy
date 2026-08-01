use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

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

    let body = Paragraph::new("Playlist screen placeholder")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" Playlist "));
    frame.render_widget(body, chunks[0]);

    draw_bottom_bar(frame, chunks[1]);
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

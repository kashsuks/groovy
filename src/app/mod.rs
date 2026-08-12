pub mod browser;

use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::KeyCode;

use crate::config::Config;
use crate::library::{self, Track};
use crate::player::{PlayerCommand, PlayerHandle, PlayerStatus};
use browser::BrowserState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Playlist,
    Cinema,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HomeMode {
    List,
    Browsing,
    Naming,
}

pub struct ActivePlaylist {
    pub name: String,
    pub path: PathBuf,
    pub tracks: Vec<Track>,
}

/// Transport state for whatevers currently loaded in the player thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

impl RepeatMode {
    fn cycle(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }
}

/// One of the five icon buttons on the persistent bottom bar. Shared between
/// keyboard handling and mouse click hit-testing so both paths trigger the
/// exact same behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlButton {
    Prev,
    PlayPause,
    Next,
    Shuffle,
    Repeat,
}

impl ControlButton {
    pub fn activate(self, app: &mut App) {
        match self {
            ControlButton::Prev => app.play_previous_track(),
            ControlButton::PlayPause => app.toggle_pause(),
            ControlButton::Next => app.advance_to_next_track(),
            ControlButton::Shuffle => app.toggle_shuffle(),
            ControlButton::Repeat => app.cycle_repeat(),
        }
    }
}

pub struct App {
    pub screen: Screen,
    pub sidebar_open: bool,
    pub should_quit: bool,

    pub config: Config,
    pub home_mode: HomeMode,
    pub home_selected: usize,
    pub browser: Option<BrowserState>,
    pub naming_input: String,
    pub status_message: Option<String>,
    pub current_playlist: Option<ActivePlaylist>,

    pub player: PlayerHandle,
    pub playback_state: PlaybackState,
    pub now_playing_index: Option<usize>,
    pub position: Duration,
    pub track_selected: usize,
    pub shuffle: bool,
    pub repeat_mode: RepeatMode,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        Self {
            screen: Screen::Home,
            sidebar_open: false,
            should_quit: false,
            config,
            home_mode: HomeMode::List,
            home_selected: 0,
            browser: None,
            naming_input: String::new(),
            status_message: None,
            current_playlist: None,
            player: PlayerHandle::spawn(),
            playback_state: PlaybackState::Stopped,
            now_playing_index: None,
            position: Duration::ZERO,
            track_selected: 0,
            shuffle: false,
            repeat_mode: RepeatMode::Off,
        }
    }

    /// Drains pending status updates from the player thread and folds them
    pub fn poll_player(&mut self) {
        for status in self.player.poll() {
            match status {
                PlayerStatus::Playing { position, .. } => {
                    self.playback_state = PlaybackState::Playing;
                    self.position = position;
                }
                PlayerStatus::Paused { position } => {
                    self.playback_state = PlaybackState::Paused;
                    self.position = position;
                }
                PlayerStatus::Stopped => {
                    self.playback_state = PlaybackState::Stopped;
                    self.now_playing_index = None;
                    self.position = Duration::ZERO;
                }
                PlayerStatus::Finished => {
                    self.playback_state = PlaybackState::Stopped;
                    self.position = Duration::ZERO;
                    self.on_track_finished();
                }
                PlayerStatus::Error(message) => {
                    self.status_message = Some(message);
                    self.playback_state = PlaybackState::Stopped;
                }
            }
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        if code == KeyCode::Char('q') && self.home_mode == HomeMode::List {
            self.should_quit = true;
            return;
        }
        match self.screen {
            Screen::Home => self.handle_home_key(code),
            Screen::Playlist | Screen::Cinema => self.handle_playback_screen_key(code),
        }
    }

    fn handle_playback_screen_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('b') => self.sidebar_open = !self.sidebar_open,
            KeyCode::Down | KeyCode::Char('j') => self.move_track_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_track_selection(-1),
            KeyCode::Char(' ') => self.toggle_pause(),
            KeyCode::Char('n') => self.advance_to_next_track(),
            KeyCode::Char('p') => self.play_previous_track(),
            KeyCode::Char('s') => self.toggle_shuffle(),
            KeyCode::Char('r') => self.cycle_repeat(),
            KeyCode::Enter => self.play_selected_track(),
            KeyCode::Esc => self.retreat_screen(),
            _ => {}
        }
    }

    pub fn handle_scroll(&mut self, delta: i32) {
        match self.screen {
            Screen::Playlist | Screen::Cinema => self.move_track_selection(delta),
            Screen::Home => {}
        }
    }

    fn move_track_selection(&mut self, delta: i32) {
        let Some(playlist) = &self.current_playlist else { return };
        if playlist.tracks.is_empty() {
            return;
        }
        let last = playlist.tracks.len() as i32 - 1;
        let next = (self.track_selected as i32 + delta).clamp(0, last);
        self.track_selected = next as usize;
    }

    fn play_selected_track(&mut self) {
        self.play_track_at(self.track_selected);
    }

    fn play_track_at(&mut self, index: usize) {
        let Some(playlist) = &self.current_playlist else { return };
        let Some(track) = playlist.tracks.get(index) else { return };
        self.player.send(PlayerCommand::Play(track.path.clone()));
        self.now_playing_index = Some(index);
    }

    /// Called when the player thread reports a track finished on its own.
    /// Repeat-one replays the same track instead of advancing.
    fn on_track_finished(&mut self) {
        if self.repeat_mode == RepeatMode::One {
            if let Some(i) = self.now_playing_index {
                self.play_track_at(i);
                return;
            }
        }
        self.advance_to_next_track();
    }

    fn advance_to_next_track(&mut self) {
        let Some(playlist) = &self.current_playlist else { return };
        let len = playlist.tracks.len();
        if len == 0 {
            return;
        }

        let next_index = if self.shuffle {
            self.random_track_index(len)
        } else {
            match self.now_playing_index {
                Some(i) if i + 1 < len => i + 1,
                Some(_) if self.repeat_mode == RepeatMode::All => 0,
                None => 0,
                _ => {
                    self.now_playing_index = None;
                    return;
                }
            }
        };
        self.play_track_at(next_index);
    }

    fn play_previous_track(&mut self) {
        let Some(playlist) = &self.current_playlist else { return };
        let len = playlist.tracks.len();
        if len == 0 {
            return;
        }

        let prev_index = if self.shuffle {
            self.random_track_index(len)
        } else {
            match self.now_playing_index {
                Some(0) if self.repeat_mode == RepeatMode::All => len - 1,
                Some(0) | None => 0,
                Some(i) => i - 1,
            }
        };
        self.play_track_at(prev_index);
    }

    /// Cheap pseudo-random pick, good enough for shuffle — not used anywhere
    /// that needs real randomness quality.
    fn random_track_index(&self, len: usize) -> usize {
        if len <= 1 {
            return 0;
        }
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as usize;
        let mut index = nanos % len;
        if Some(index) == self.now_playing_index {
            index = (index + 1) % len;
        }
        index
    }

    fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }

    fn cycle_repeat(&mut self) {
        self.repeat_mode = self.repeat_mode.cycle();
    }

    /// Toggles pause/resume. No-op if nothing is currently loaded.
    fn toggle_pause(&mut self) {
        match self.playback_state {
            PlaybackState::Playing => self.player.send(PlayerCommand::Pause),
            PlaybackState::Paused => self.player.send(PlayerCommand::Resume),
            PlaybackState::Stopped => {}
        }
    }
    fn handle_home_key(&mut self, code: KeyCode) {
        match self.home_mode {
            HomeMode::List => self.handle_home_list_key(code),
            HomeMode::Browsing => self.handle_browsing_key(code),
            HomeMode::Naming => self.handle_naming_key(code),
        }
    }
    fn handle_home_list_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('n') => {
                let start = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                self.browser = Some(BrowserState::new(start));
                self.home_mode = HomeMode::Browsing;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.config.playlists.is_empty() {
                    self.home_selected =
                        (self.home_selected + 1).min(self.config.playlists.len() - 1);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.home_selected = self.home_selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(entry) = self.config.playlists.get(self.home_selected) {
                    let tracks = library::scan_playlist(&entry.path);
                    self.current_playlist = Some(ActivePlaylist {
                        name: entry.name.clone(),
                        path: entry.path.clone(),
                        tracks,
                    });
                    self.track_selected = 0;
                    self.screen = Screen::Playlist;
                }
            }
            _ => {}
        }
    } // n = new playlist
    fn handle_browsing_key(&mut self, code: KeyCode) {
        let Some(browser) = &mut self.browser else { return };

        if browser.popup.is_some() {
            match code {
                KeyCode::Esc => browser.close_popup(),
                KeyCode::Enter => browser.popup_confirm(),
                KeyCode::Backspace => browser.popup_backspace(),
                KeyCode::Char(c) => browser.popup_push_char(c),
                _ => {}
            }
            return;
        }

        match code {
            KeyCode::Esc => {
                self.browser = None;
                self.home_mode = HomeMode::List;
            }
            KeyCode::Down | KeyCode::Char('j') => browser.move_down(),
            KeyCode::Up | KeyCode::Char('k') => browser.move_up(),
            KeyCode::Enter | KeyCode::Char('l') => browser.enter_selected(),
            KeyCode::Backspace | KeyCode::Char('h') => browser.go_to_parent(),
            KeyCode::Char('/') => browser.open_popup(),
            KeyCode::Char('s') => {
                self.naming_input.clear();
                self.home_mode = HomeMode::Naming;
            }
            _ => {}
        }
    }
    fn handle_naming_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.home_mode = HomeMode::Browsing;
            }
            KeyCode::Backspace => {
                self.naming_input.pop();
            }
            KeyCode::Char(c) => self.naming_input.push(c),
            KeyCode::Enter if !self.naming_input.trim().is_empty() => {
                if let Some(browser) = &self.browser {
                    let name = self.naming_input.trim().to_string();
                    let path = browser.current_dir.clone();
                    match self.config.add_playlist(name, path) {
                        Ok(()) => self.status_message = None,
                        Err(e) => self.status_message = Some(format!("save failed: {e}")),
                    }
                }
                self.browser = None;
                self.naming_input.clear();
                self.home_mode = HomeMode::List;
            }
            _ => {}
        }
    }
    fn advance_screen(&mut self) {
        self.screen = match self.screen {
            Screen::Home => Screen::Playlist,
            Screen::Playlist => Screen::Cinema,
            Screen::Cinema => Screen::Cinema,
        };
    }
    fn retreat_screen(&mut self) {
        self.screen = match self.screen {
            Screen::Home => Screen::Home,
            Screen::Playlist => Screen::Home,
            Screen::Cinema => Screen::Playlist,
        };
    }
}

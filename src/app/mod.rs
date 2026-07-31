pub mod browser;

use std::path::PathBuf;

use crossterm::event::KeyCode;

use crate::config::Config;
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
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        if code == KeyCode::Char('q') && self.home_mode == HomeMode::List {
            self.should_quit = true;
            return;
        }
        match self.screen {
            Screen::Home => self.handle_home_key(code),
            Screen::Playlist | Screen::Cinema => self.handle_playback_screen(code),
        }
    }

    fn handle_playback_screen_key(&mut self, code: KeyCode) { ... }
    fn handle_home_key(&mut self, code: KeyCode) { ... }
    fn handle_home_list_key(&mut self, code: KeyCode) { ... } // n = new playlist
                                                    fn handle_browsing_key(&mut self, code: KeyCode) { ... }
                                                    fn handle_naming_key(&mut self, code: KeyCode) { ... }
                                                    fn advance_screen(&mut self) { ... }
                                                    fn retreat_screen(&mut self) { ... }
}

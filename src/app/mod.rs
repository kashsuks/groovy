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
            Screen::Playlist | Screen::Cinema => self.handle_playback_screen_key(code),
        }
    }

    fn handle_playback_screen_key(&mut self, code: KeyCode) { 
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('b') => self.sidebar_open = !self.sidebar_open,
            KeyCode::Enter => self.advance_screen(),
            KeyCode::Esc => self.retreat_screen(),
            _ => {}
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
                if self.config.playlists.get(self.home_selected).is_some() {
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
            KeyCode::Enter => {
                if !self.naming_input.trim().is_empty() {
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

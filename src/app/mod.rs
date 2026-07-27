use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Playlist,
    Cinema,
}

pub struct App {
    pub screen: Screen,
    pub sidebar_open: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Home,
            sidebar_open: false,
            should_quit: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q')  => self.should_quit = true,
            KeyCode::Char('b') if self.screen != Screen::Home => {
                self.sidebar_open = !self.sidebar_open;
            }
            KeyCode::Enter => self.advance_screen(),
            KeyCode::Esc => self.retreat_screen(),
            _ => {}
        }
    }

    fn advance_screen(&mut self) {
        self.screen = match self.screen {
            Screen::Home => Screen::Playlist,
            Screen::Playlist => Screen::Cinema,
            Screen::Cinema => Screen::Cinema,
        }
    }

    fn retreat_screen(&mut self) {
        self.screen = match self.screen {
            Screen::Home => Screen::Home,
            Screen::Playlist => Screen::Home,
            Screen::Cinema => Screen::Playlist,
        };
    }
}

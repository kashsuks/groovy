use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DirEntryItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

/// Whether the browsers `/` popup is currently open
#[derive(Debug, Default)]
pub struct PathPopup {
    pub input: String,
}

pub struct BrowserState {
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntryItem>,
    pub selected: usize,
    pub popup: Option<PathPopup>,
}

impl BrowserState {
    /// Stats browsing at the given directory
    pub fn new(start_dir: PathBuf) -> Self {
        let mut state = Self {
            current_dir: start_dir,
            entries: Vec::new(),
            selected: 0,
            popup: None,
        };
        state.refresh();
        state
    }

    pub fn refresh(&mut self) {
        let mut entries: Vec<DirEntryItem> = fs::read_dir(&self.current_dir)
            .map(|read_dir| {
                read_dir
                    .filter_map(|entry| entry.ok())
                    .filter_map(|entry| {
                        let path = entry.path();
                        let is_dir = entry.file_type().ok()?.is_dir();
                        let name = entry.file_name().to_string_lossy().into_owned();
                        Some(DirEntryItem { name, path, is_dir })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // directories first, then alphabetical
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        self.entries = entries;
        self.selected = 0;
    }

    pub fn move_down(&mut self) {
        if !self.entries.is_empty() {
            self.selected = (self.selected + 1).min(self.entries.len() - 1);
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn enter_selected(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                self.current_dir = entry.path.clone();
                self.refresh();
            }
        }
    }

    pub fn go_to_parent(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh();
        }
    }

    pub fn open_popup(&mut self) {
        self.popup = Some(PathPopup::default());
    }

    pub fn close_popup(&mut self) {
        self.popup = None;
    }

    pub fn popup_push_char(&mut self, c: char) {
        if let Some(popup) = &mut self.popup {
            popup.input.push(c);
        }
    }

    pub fn popup_backspace(&mut self) {
        if let Some(popup) = &mut self.popup {
            popup.input.pop();
        }
    }

    /// Navigates the browser to the typed path iif its a valid path
    pub fn popup_confirm(&mut self) {
        let Some(popup) = &self.popup else { return };
        let typed = PathBuf::from(popup.input.trim());

        if typed.is_dir() {
            self.current_dir = typed;
            self.refresh();
            self.popup = None;
        }
        // else: leave popup open with invalid input
    }
}

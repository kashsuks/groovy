use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistEntry {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub playlists: Vec<PlaylistEntry>,
}

impl Config {
    /// Resolves to per-os config directory in pair with `groove/config.toml`
    fn path() -> color_eyre::Result<PathBuf> {
        let dir = dirs::config_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("could not determine config directory"))?;
        Ok(dir.join("groove").join("config.toml"))
    }

    /// Loads config from disk, returning any empty default config if no file
    pub fn load() -> color_eyre::Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> color_eyre::Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    pub fn add_playlist(&mut self, name: String, path: PathBuf) -> color_eyre::Result<()> {
        self.playlists.push(PlaylistEntry { name, path });
        self.save()
    }
}

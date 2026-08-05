use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Play(PathBuf),
    Pause,
    Resume,
    Stop,
}

/// Status updates sent from the player thread back to the UI thread
#[derive(Debug, Clone)]
pub enum PlayerStatus {
    Playing { path: PathBuf, position: Duration },
    Paused { position: Duration },
    /// The current track finished on its own (not a manual stop)
    Finished,
    Stopped,
    Error(String),
}

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use rodio::{Decoder, OutputStream, Sink};

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

pub struct PlayerHandle {
    command_tx: Sender<PlayerCommand>,
    status_rx: Receiver<PlayerStatus>,
}

impl PlayerHandle {
    pub fn spawn() -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (status_tx, status_rx) = mpsc::channel();

        thread::spawn(move || run_player_thread(command_rx, status_tx));

        Self { command_tx, status_rx }
    }

    pub fn send(&self, command: PlayerCommand) {
        let _ = self.command_tx.send(command);
    }

    pub fn poll(&self) -> Vec<PlayerStatus> {
        self.status_rx.try_iter().collect()
    }
}

fn run_player_thread(command_rx: Receiver<PlayerCommand>, status_tx: Sender<PlayerStatus>) {
    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok(pair) => pair,
        Err(e) => {
            let _ = status_tx.send(PlayerStatus::Error(format!("no audio output device: {e}")));
            return;
        }
    };

    let mut sink: Option<Sink> = None;
    let mut current_path: Option<PathBuf> = None;
    let mut started_at: Option<Instant> = None;
    let mut paused_position: Duration = Duration::ZERO;

    loop {
        match command_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(PlayerCommand::Play(path)) => {
                match play_file(&stream_handle, &path) {
                    Ok(new_sink) => {
                        sink = Some(new_sink);
                        current_path = Some(path.clone());
                        started_at = Some(Instant::now());
                        paused_position = Duration::ZERO;
                        let _ = status_tx.send(PlayerStatus::Playing { path, position: Duration::ZERO });
                    }
                    Err(e) => {
                        let _ = status_tx.send(PlayerStatus::Error(format!("couldn't play {}: {e}", path.display())));
                    }
                }
            }
            Ok(PlayerCommand::Pause) => {
                if let Some(s) = &sink {
                    s.pause();
                    let position = elapsed_position(started_at, paused_position);
                    paused_position = position;
                    started_at = None;
                    let _ = status_tx.send(PlayerStatus::Paused { position });
                }
            }
            Ok(PlayerCommand::Resume) => {
                if let Some(s) = &sink {
                    s.play();
                    started_at = Some(Instant::now());
                    if let Some(path) = &current_path {
                        let _ = status_tx.send(PlayerStatus::Playing { path: path.clone(), position: paused_position });
                    }
                }
            }
            Ok(PlayerCommand::Stop) => {
                sink = None;
                current_path = None;
                started_at = None;
                paused_position = Duration::ZERO;
                let _ = status_tx.send(PlayerStatus::Stopped);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(s) = &sink {
                    if s.empty() {
                        sink = None;
                        current_path = None;
                        started_at = None;
                        let _ = status_tx.send(PlayerStatus::Finished);
                    } else if started_at.is_some() {
                        // Actually playing (not paused) — the UI needs a
                        // fresh position every tick to advance the progress bar.
                        if let Some(path) = &current_path {
                            let position = elapsed_position(started_at, paused_position);
                            let _ = status_tx.send(PlayerStatus::Playing { path: path.clone(), position });
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn play_file(stream_handle: &rodio::OutputStreamHandle, path: &PathBuf) -> color_eyre::Result<Sink> {
    let sink = Sink::try_new(stream_handle)?;
    let file = File::open(path)?;
    let source = Decoder::new(BufReader::new(file))?;
    sink.append(source);
    Ok(sink)
}

fn elapsed_position(started_at: Option<Instant>, paused_position: Duration) -> Duration {
    match started_at {
        Some(start) => paused_position + start.elapsed(),
        None => paused_position,
    }
}

use std::path::{Path, PathBuf};
use std::time::Duration;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Track {
    /// 1-based position in the playlists flattened order
    pub index: usize,
    pub title: String,
    pub path: PathBuf,
    pub duration: Duration,
}

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus", "wma"];

fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| AUDIO_EXTENSIONS.contains(&&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Recursively scan a playlist folder for audio files. travesal is 
/// dfs, alphabetical within in each directory (via `sort_by_file_name`)
///
/// File that fail to read (corrupt tags, unsupported codec details, etc.)
/// are skipped rather than aborting the whole scan as one bad file shouldnt block the rest from
/// loading
pub fn scan_playlist(root: &Path) -> Vec<Track> {
    let mut tracks = Vec::new();

    for entry in WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !entry.file_type().is_file() || !is_audio_file(path) {
            continue;
        }

        if let Some(track) = read_track(path) {
            tracks.push(track);
        }
    }

    for (i, track) in tracks.iter_mut().enumerate() {
        track.index = i + 1;
    }

    tracks
}

fn read_track(path: &Path) -> Option<Track> {
    let tagged_file = Probe::open(path).ok()?.read().ok()?;

    let title = tagged_file
        .primary_tag()
        .and_then(|tag| tag.title())
        .map(|t| t.to_string())
        .unwrap_or_else(|| {
            path.file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Unknown".to_string())
        });

    let duration = tagged_file.properties().duration();

    Some(Track {
        index: 0, // assigned by scan_playlist after the full traversal
        title,
        path: path.to_path_buf(),
        duration,
    })
}

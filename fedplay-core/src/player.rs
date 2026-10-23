//! Estados do player

/// Estado de reprodução
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Estado completo do player
#[derive(Debug, Clone)]
pub struct PlayerState {
    pub playback: PlaybackState,
    pub current_track_id: Option<u64>,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub volume: f64,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            playback: PlaybackState::Stopped,
            current_track_id: None,
            position_ms: 0,
            duration_ms: 0,
            volume: 1.0,
        }
    }
}
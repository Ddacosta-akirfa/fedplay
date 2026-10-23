//! Fedplay Core - Lógica de negócio e estruturas de dados
//! 
//! Este módulo contém as estruturas fundamentais do Fedplay,
//! independentes de UI e backend de áudio.

pub mod track;
pub mod player;

pub use track::{Track, TrackMetadata};
pub use player::{PlayerState, PlaybackState};
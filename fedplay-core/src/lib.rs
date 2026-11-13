//! Fedplay Core - Lógica de negócio e estruturas de dados
//!
//! Este módulo contém as estruturas fundamentais do Fedplay,
//! independentes de UI e backend de áudio.

pub mod library;
pub mod metadata;
pub mod player;
pub mod track;

pub use library::{DirectoryScanner, ScanStats, ScannerError};
pub use metadata::{MetadataError, read_metadata, write_metadata};
pub use player::{PlaybackState, PlayerState};
pub use track::{Track, TrackMetadata};

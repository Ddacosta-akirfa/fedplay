//! Estruturas de dados para faixas de áudio

use std::path::PathBuf;

/// Metadados de uma faixa de áudio
#[derive(Debug, Clone, Default)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Representação de uma faixa de áudio
#[derive(Debug, Clone)]
pub struct Track {
    pub id: u64,
    pub path: PathBuf,
    pub metadata: TrackMetadata,
}

impl Track {
    /// Cria uma nova faixa
    pub fn new(id: u64, path: PathBuf, metadata: TrackMetadata) -> Self {
        Self {
            id,
            path,
            metadata,
        }
    }

    /// Retorna o título da faixa ou o nome do arquivo
    pub fn display_title(&self) -> String {
        self.metadata
            .title
            .clone()
            .or_else(|| {
                self.path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "Faixa desconhecida".to_string())
    }

    /// Retorna o artista da faixa
    pub fn display_artist(&self) -> String {
        self.metadata
            .artist
            .clone()
            .unwrap_or_else(|| "Artista desconhecido".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_display_title() {
        let track = Track::new(
            1,
            PathBuf::from("/music/test.mp3"),
            TrackMetadata {
                title: Some("Minha Música".to_string()),
                ..Default::default()
            },
        );

        assert_eq!(track.display_title(), "Minha Música");
    }

    #[test]
    fn test_track_display_title_fallback() {
        let track = Track::new(
            1,
            PathBuf::from("/music/minha_musica.mp3"),
            TrackMetadata::default(),
        );

        assert_eq!(track.display_title(), "minha_musica");
    }
}
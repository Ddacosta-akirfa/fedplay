//! Manipulação de metadados de áudio usando lofty-rs

use crate::TrackMetadata;
use lofty::{Accessor, AudioFile, TaggedFileExt};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Falha ao ler o arquivo de áudio: {0}")]
    ReadError(String),
    #[error("Falha ao salvar o arquivo de áudio: {0}")]
    SaveError(String),
    #[error("Nenhuma tag encontrada no arquivo")]
    NoTagsFound,
    #[error(
        "Codificação de texto incompatível nas tags (ex: Latin-1). Tente limpar as tags com outro programa primeiro."
    )]
    EncodingError,
}

/// Lê os metadados de um arquivo de áudio
pub fn read_metadata(path: &Path) -> Result<TrackMetadata, MetadataError> {
    let tagged_file = match lofty::read_from_path(path) {
        Ok(file) => file,
        Err(e) => {
            // Se o erro for de decodificação de texto, retornamos nosso erro customizado
            if e.to_string().contains("UTF-8") || e.to_string().contains("decoding") {
                return Err(MetadataError::EncodingError);
            }
            // Caso contrário, retornamos um metadata vazio baseado no nome do arquivo,
            // permitindo que o usuário pelo menos veja o nome e tente sobrescrever.
            return Ok(TrackMetadata {
                title: path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string()),
                ..Default::default()
            });
        }
    };

    let tag = tagged_file
        .primary_tag()
        .ok_or(MetadataError::NoTagsFound)?;
    let duration_ms = tagged_file.properties().duration().as_millis() as u64;

    Ok(TrackMetadata {
        title: tag.title().map(|s| s.to_string()),
        artist: tag.artist().map(|s| s.to_string()),
        album: tag.album().map(|s| s.to_string()),
        track_number: tag.track().map(|t| t as u32),
        year: tag.year().map(|y| y as u32),
        genre: tag.genre().map(|s| s.to_string()),
        duration_ms: Some(duration_ms),
    })
}

/// Salva os metadados em um arquivo de áudio
pub fn write_metadata(path: &Path, metadata: &TrackMetadata) -> Result<(), MetadataError> {
    let mut tagged_file =
        lofty::read_from_path(path).map_err(|e| MetadataError::ReadError(e.to_string()))?;

    let tag = tagged_file
        .primary_tag_mut()
        .ok_or(MetadataError::NoTagsFound)?;

    if let Some(ref title) = metadata.title {
        tag.set_title(title.clone());
    }
    if let Some(ref artist) = metadata.artist {
        tag.set_artist(artist.clone());
    }
    if let Some(ref album) = metadata.album {
        tag.set_album(album.clone());
    }
    if let Some(track) = metadata.track_number {
        tag.set_track(track);
    }
    if let Some(year) = metadata.year {
        tag.set_year(year as u32);
    }
    if let Some(ref genre) = metadata.genre {
        tag.set_genre(genre.clone());
    }

    tagged_file
        .save_to_path(path)
        .map_err(|e| MetadataError::SaveError(e.to_string()))?;

    Ok(())
}

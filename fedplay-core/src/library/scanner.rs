use crate::{Track, TrackMetadata, read_metadata};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Extensões de áudio suportadas pelo FedPlay.
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "opus", "m4a", "aac", "wma"];

/// Erros que podem ocorrer durante a leitura de diretórios.
#[derive(Debug, Error)]
pub enum ScannerError {
    #[error("O caminho não é um diretório válido: {0}")]
    InvalidDirectory(PathBuf),

    #[error("Erro ao aceder ao diretório '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Resultado da leitura de uma pasta.
#[derive(Debug, Default)]
pub struct ScanStats {
    pub tracks: Vec<Track>,
    pub files_scanned: usize,
    pub unsupported_files: usize,
}

/// Responsável por procurar ficheiros de áudio nos diretórios.
#[derive(Debug, Default)]
pub struct DirectoryScanner;

impl DirectoryScanner {
    pub fn new() -> Self {
        Self
    }

    /// Percorre um diretório e todas as suas subpastas.
    pub fn scan_directory(&self, path: &Path) -> Result<ScanStats, ScannerError> {
        if !path.is_dir() {
            return Err(ScannerError::InvalidDirectory(path.to_path_buf()));
        }

        let mut stats = ScanStats::default();

        self.scan_recursive(path, &mut stats)?;

        Ok(stats)
    }

    /// Percorre recursivamente os diretórios.
    fn scan_recursive(&self, directory: &Path, stats: &mut ScanStats) -> Result<(), ScannerError> {
        let entries = fs::read_dir(directory).map_err(|source| ScannerError::Io {
            path: directory.to_path_buf(),
            source,
        })?;

        for entry in entries {
            let entry = entry.map_err(|source| ScannerError::Io {
                path: directory.to_path_buf(),
                source,
            })?;

            let path = entry.path();

            let file_type = entry.file_type().map_err(|source| ScannerError::Io {
                path: path.clone(),
                source,
            })?;

            // Ignora links simbólicos para evitar ciclos.
            if file_type.is_symlink() {
                continue;
            }

            if file_type.is_dir() {
                self.scan_recursive(&path, stats)?;
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            if Self::is_audio_file(&path) {
                stats.files_scanned += 1;

                let metadata = Self::extract_metadata(&path);

                let track = Track::new(0, path, metadata);

                stats.tracks.push(track);
            } else {
                stats.unsupported_files += 1;
            }
        }

        Ok(())
    }

    /// Verifica se o ficheiro possui uma extensão suportada.
    fn is_audio_file(path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| AUDIO_EXTENSIONS.contains(&extension.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Extrai metadados ou utiliza o nome do ficheiro como título.
    fn extract_metadata(path: &Path) -> TrackMetadata {
        match read_metadata(path) {
            Ok(mut metadata) => {
                if metadata.title.is_none() {
                    metadata.title = Self::file_title(path);
                }

                metadata
            }
            Err(_) => TrackMetadata {
                title: Self::file_title(path),
                ..Default::default()
            },
        }
    }

    /// Obtém o nome do ficheiro sem a extensão.
    fn file_title(path: &Path) -> Option<String> {
        path.file_stem()
            .map(|name| name.to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_directory() -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let path =
            std::env::temp_dir().join(format!("fedplay-test-{}-{timestamp}", std::process::id()));

        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn test_scan_directory_recursively() {
        let root = create_test_directory();

        let rock = root.join("Rock");
        let jazz = root.join("Jazz");

        fs::create_dir_all(&rock).unwrap();
        fs::create_dir_all(&jazz).unwrap();

        fs::write(root.join("music.mp3"), b"fake audio").unwrap();
        fs::write(rock.join("guitar.FLAC"), b"fake audio").unwrap();
        fs::write(jazz.join("piano.wav"), b"fake audio").unwrap();
        fs::write(root.join("cover.jpg"), b"fake image").unwrap();
        fs::write(root.join("notes.txt"), b"text").unwrap();

        let scanner = DirectoryScanner::new();
        let result = scanner.scan_directory(&root).unwrap();

        assert_eq!(result.files_scanned, 3);
        assert_eq!(result.unsupported_files, 2);
        assert_eq!(result.tracks.len(), 3);

        fs::remove_dir_all(root).unwrap();
    }
}

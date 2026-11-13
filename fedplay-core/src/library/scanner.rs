use crate::{Track, TrackMetadata, read_metadata};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use thiserror::Error;

/// Extensões de áudio suportadas pelo FedPlay.
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "opus", "m4a", "aac", "wma"];

#[derive(Debug, Error)]
pub enum ScannerError {
    #[error("O caminho não é um diretório válido: {0}")]
    InvalidDirectory(PathBuf),

    #[error("Erro ao aceder ao diretório ou ficheiro '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Não foi possível obter a data de modificação de: {0}")]
    InvalidModificationTime(PathBuf),

    #[error("O tamanho do ficheiro excede o limite suportado: {0}")]
    FileTooLarge(PathBuf),
}

/// Representa um ficheiro de áudio descoberto, ainda sem ler os seus metadados.
#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub file_size: i64,
    pub file_mtime: i64,
}

/// Resultado da descoberta dos ficheiros.
#[derive(Debug, Default)]
pub struct FileScanStats {
    pub files: Vec<ScannedFile>,
    pub files_scanned: usize,
    pub unsupported_files: usize,
}

/// Resultado da leitura dos ficheiros com os seus metadados.
#[derive(Debug, Default)]
pub struct ScanStats {
    pub tracks: Vec<Track>,
    pub files_scanned: usize,
    pub unsupported_files: usize,
}

#[derive(Debug, Default)]
pub struct DirectoryScanner;

impl DirectoryScanner {
    pub fn new() -> Self {
        Self
    }

    /// Descobre ficheiros de áudio sem extrair os seus metadados.
    pub fn scan_directory_files(&self, path: &Path) -> Result<FileScanStats, ScannerError> {
        if !path.is_dir() {
            return Err(ScannerError::InvalidDirectory(path.to_path_buf()));
        }

        let mut stats = FileScanStats::default();
        self.scan_files_recursive(path, &mut stats)?;

        Ok(stats)
    }

    /// Mantém a operação original: descobre ficheiros e extrai metadados.
    pub fn scan_directory(&self, path: &Path) -> Result<ScanStats, ScannerError> {
        let file_scan = self.scan_directory_files(path)?;

        let tracks = file_scan
            .files
            .into_iter()
            .map(|file| {
                let metadata = Self::extract_metadata(&file.path);
                Track::new(0, file.path, metadata)
            })
            .collect();

        Ok(ScanStats {
            tracks,
            files_scanned: file_scan.files_scanned,
            unsupported_files: file_scan.unsupported_files,
        })
    }

    fn scan_files_recursive(
        &self,
        directory: &Path,
        stats: &mut FileScanStats,
    ) -> Result<(), ScannerError> {
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
                self.scan_files_recursive(&path, stats)?;
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            if Self::is_audio_file(&path) {
                let metadata = fs::metadata(&path).map_err(|source| ScannerError::Io {
                    path: path.clone(),
                    source,
                })?;

                let file_size = i64::try_from(metadata.len())
                    .map_err(|_| ScannerError::FileTooLarge(path.clone()))?;

                let modified = metadata.modified().map_err(|source| ScannerError::Io {
                    path: path.clone(),
                    source,
                })?;

                let file_mtime = modified
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| ScannerError::InvalidModificationTime(path.clone()))?
                    .as_nanos();

                let file_mtime = i64::try_from(file_mtime)
                    .map_err(|_| ScannerError::InvalidModificationTime(path.clone()))?;

                stats.files.push(ScannedFile {
                    path,
                    file_size,
                    file_mtime,
                });

                stats.files_scanned += 1;
            } else {
                stats.unsupported_files += 1;
            }
        }

        Ok(())
    }

    fn is_audio_file(path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| AUDIO_EXTENSIONS.contains(&extension.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    pub(crate) fn extract_metadata(path: &Path) -> TrackMetadata {
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

    #[test]
    fn test_scan_directory_files_collects_file_information() {
        let root = create_test_directory();
        let song = root.join("music.mp3");

        fs::write(&song, b"fake audio").unwrap();

        let scanner = DirectoryScanner::new();
        let result = scanner.scan_directory_files(&root).unwrap();

        assert_eq!(result.files_scanned, 1);
        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].path, song);
        assert_eq!(result.files[0].file_size, 10);
        assert!(result.files[0].file_mtime > 0);

        fs::remove_dir_all(root).unwrap();
    }
}

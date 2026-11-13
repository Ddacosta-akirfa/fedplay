use crate::{Database, DirectoryScanner, ScannerError, Track};
use rusqlite::{Transaction, params};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error(transparent)]
    Scanner(#[from] ScannerError),

    #[error(transparent)]
    Database(#[from] rusqlite::Error),
}

#[derive(Debug, Default)]
pub struct ImportSummary {
    pub tracks_found: usize,
    pub tracks_saved: usize,
    pub ignored_files: usize,
}

pub struct LibraryManager {
    database: Database,
    scanner: DirectoryScanner,
}

impl LibraryManager {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            scanner: DirectoryScanner::new(),
        }
    }

    pub fn import_directory(&self, path: &Path) -> Result<ImportSummary, LibraryError> {
        // 1. Descobrir as músicas.
        let scan = self.scanner.scan_directory(path)?;

        let now = current_timestamp();
        let directory_path = path.to_string_lossy().into_owned();

        // 2. Iniciar uma transação.
        let transaction = self.database.connection().unchecked_transaction()?;

        // 3. Registar o diretório monitorizado.
        transaction.execute(
            "INSERT OR IGNORE INTO watched_directories (path, created_at)
             VALUES (?1, ?2)",
            params![directory_path, now],
        )?;

        let directory_id: i64 = transaction.query_row(
            "SELECT id FROM watched_directories WHERE path = ?1",
            params![directory_path],
            |row| row.get(0),
        )?;

        transaction.execute(
            "UPDATE watched_directories
             SET last_scan = ?1
             WHERE id = ?2",
            params![now, directory_id],
        )?;

        // 4. Guardar cada faixa.
        for track in &scan.tracks {
            Self::save_track(&transaction, track, directory_id, now)?;
        }

        // 5. Confirmar todas as operações.
        transaction.commit()?;

        Ok(ImportSummary {
            tracks_found: scan.tracks.len(),
            tracks_saved: scan.tracks.len(),
            ignored_files: scan.unsupported_files,
        })
    }

    fn save_track(
        transaction: &Transaction<'_>,
        track: &Track,
        directory_id: i64,
        now: i64,
    ) -> Result<(), rusqlite::Error> {
        let metadata = &track.metadata;

        let artist_name = metadata
            .artist
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("Artista Desconhecido");

        let album_title = metadata
            .album
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("Álbum Desconhecido");

        let artist_id = Self::get_or_create_artist(transaction, artist_name, now)?;

        let album_id = Self::get_or_create_album(
            transaction,
            album_title,
            artist_id,
            metadata.year.map(i64::from),
            now,
        )?;

        let file_path = track.path.to_string_lossy().into_owned();

        transaction.execute(
            "INSERT INTO tracks (
                path, directory_id, artist_id, album_id,
                title, track_number, genre, duration_ms,
                created_at, updated_at
            )
            VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9
            )
            ON CONFLICT(path) DO UPDATE SET
                directory_id = excluded.directory_id,
                artist_id = excluded.artist_id,
                album_id = excluded.album_id,
                title = excluded.title,
                track_number = excluded.track_number,
                genre = excluded.genre,
                duration_ms = excluded.duration_ms,
                missing_since = NULL,
                updated_at = excluded.updated_at",
            params![
                file_path,
                directory_id,
                artist_id,
                album_id,
                metadata.title,
                metadata.track_number.map(i64::from),
                metadata.genre,
                metadata.duration_ms.and_then(|v| i64::try_from(v).ok()),
                now,
            ],
        )?;

        Ok(())
    }

    fn get_or_create_artist(
        transaction: &Transaction<'_>,
        name: &str,
        now: i64,
    ) -> Result<i64, rusqlite::Error> {
        transaction.execute(
            "INSERT OR IGNORE INTO artists (
                name, name_sort, created_at, updated_at
            ) VALUES (?1, ?1, ?2, ?2)",
            params![name, now],
        )?;

        transaction.query_row(
            "SELECT id FROM artists WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )
    }

    fn get_or_create_album(
        transaction: &Transaction<'_>,
        title: &str,
        artist_id: i64,
        year: Option<i64>,
        now: i64,
    ) -> Result<i64, rusqlite::Error> {
        transaction.execute(
            "INSERT OR IGNORE INTO albums (
                title, artist_id, year, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![title, artist_id, year, now],
        )?;

        let album_id: i64 = transaction.query_row(
            "SELECT id FROM albums
             WHERE title = ?1 AND artist_id = ?2",
            params![title, artist_id],
            |row| row.get(0),
        )?;

        // Não substituir um ano que já esteja registado.
        transaction.execute(
            "UPDATE albums
             SET year = COALESCE(year, ?1),
                 updated_at = ?2
             WHERE id = ?3",
            params![year, now, album_id],
        )?;

        Ok(album_id)
    }

    pub fn database(&self) -> &Database {
        &self.database
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_directory() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let path = std::env::temp_dir().join(format!(
            "fedplay-manager-test-{}-{timestamp}",
            std::process::id()
        ));

        fs::create_dir_all(&path).unwrap();
        path
    }

    fn create_manager() -> LibraryManager {
        let database = Database::new(":memory:").unwrap();
        LibraryManager::new(database)
    }

    fn count_records(manager: &LibraryManager, table: &str) -> i64 {
        // Os nomes das tabelas são constantes definidas nos testes.
        manager
            .database()
            .connection()
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap()
    }

    #[test]
    fn test_import_directory_saves_tracks() {
        let directory = create_test_directory();

        fs::write(directory.join("song-one.mp3"), b"fake audio").unwrap();
        fs::write(directory.join("song-two.flac"), b"fake audio").unwrap();

        let manager = create_manager();

        let summary = manager.import_directory(&directory).unwrap();

        assert_eq!(summary.tracks_found, 2);
        assert_eq!(summary.tracks_saved, 2);

        assert_eq!(count_records(&manager, "tracks"), 2);
        assert_eq!(count_records(&manager, "artists"), 1);
        assert_eq!(count_records(&manager, "albums"), 1);
        assert_eq!(count_records(&manager, "watched_directories"), 1);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn test_import_directory_does_not_duplicate_tracks() {
        let directory = create_test_directory();

        fs::write(directory.join("song.mp3"), b"fake audio").unwrap();

        let manager = create_manager();

        manager.import_directory(&directory).unwrap();
        manager.import_directory(&directory).unwrap();

        assert_eq!(count_records(&manager, "tracks"), 1);
        assert_eq!(count_records(&manager, "artists"), 1);
        assert_eq!(count_records(&manager, "albums"), 1);
        assert_eq!(count_records(&manager, "watched_directories"), 1);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn test_import_directory_persists_track_metadata() {
        let directory = create_test_directory();

        fs::write(directory.join("my-favorite-song.mp3"), b"fake audio").unwrap();

        let manager = create_manager();

        manager.import_directory(&directory).unwrap();

        let (title, artist, album): (String, String, String) = manager
            .database()
            .connection()
            .query_row(
                "SELECT title, artist_name, album_title
                 FROM v_tracks",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(title, "my-favorite-song");
        assert_eq!(artist, "Artista Desconhecido");
        assert_eq!(album, "Álbum Desconhecido");

        fs::remove_dir_all(directory).unwrap();
    }
}

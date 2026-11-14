use crate::TrackMetadata;
use crate::{Database, DirectoryScanner, ScannedFile, ScannerError, Track};
use rusqlite::{OptionalExtension, Transaction, params};
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
    pub tracks_added: usize,
    pub tracks_updated: usize,
    pub tracks_unchanged: usize,
    pub tracks_restored: usize,
    pub ignored_files: usize,
}

#[derive(Debug)]
struct ExistingTrack {
    file_size: Option<i64>,
    file_mtime: Option<i64>,
    is_missing: bool,
}

pub struct LibraryManager {
    database: Database,
    scanner: DirectoryScanner,
}

impl LibraryManager {
    pub fn get_tracks(&self) -> Result<Vec<Track>, LibraryError> {
        let connection = self.database.connection();

        let mut statement = connection.prepare(
            "
        SELECT
            id,
            path,
            title,
            artist_name,
            album_title,
            track_number,
            year,
            genre,
            duration_ms
        FROM v_tracks
        WHERE missing_since IS NULL
        ORDER BY
            artist_name COLLATE NOCASE,
            album_title COLLATE NOCASE,
            track_number,
            title COLLATE NOCASE
        ",
        )?;

        let tracks = statement
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let path: String = row.get(1)?;

                let metadata = map_track_metadata(row)?;

                Ok(Track::new(
                    u64::try_from(id).unwrap_or_default(),
                    Path::new(&path).to_path_buf(),
                    metadata,
                ))
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;

        Ok(tracks)
    }

    pub fn new(database: Database) -> Self {
        Self {
            database,
            scanner: DirectoryScanner::new(),
        }
    }

    pub fn import_directory(&self, path: &Path) -> Result<ImportSummary, LibraryError> {
        // 1. Descobrir ficheiros sem extrair metadados.
        let scan = self.scanner.scan_directory_files(path)?;

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

        let mut summary = ImportSummary {
            tracks_found: scan.files.len(),
            ignored_files: scan.unsupported_files,
            ..Default::default()
        };

        // 4. Processar apenas ficheiros novos ou alterados.
        for file in &scan.files {
            let existing = Self::find_existing_file(&transaction, file)?;

            match existing {
                None => {
                    Self::process_track(&self.scanner, &transaction, file, directory_id, now)?;
                    summary.tracks_added += 1;
                }

                Some(existing)
                    if existing.file_size == Some(file.file_size)
                        && existing.file_mtime == Some(file.file_mtime) =>
                {
                    Self::restore_unchanged_track(&transaction, file, directory_id, now)?;

                    if existing.is_missing {
                        // A faixa existia, desapareceu e agora foi restaurada.
                        summary.tracks_restored += 1;
                    } else {
                        summary.tracks_unchanged += 1;
                    }
                }

                Some(_) => {
                    Self::process_track(&self.scanner, &transaction, file, directory_id, now)?;
                    summary.tracks_updated += 1;
                }
            }
        }

        // 5. Marcar como ausentes os ficheiros que desapareceram.
        Self::mark_missing_tracks(&transaction, directory_id, now, &scan.files)?;

        // 6. Confirmar todas as operações.
        transaction.commit()?;

        summary.tracks_saved = summary.tracks_added + summary.tracks_updated;

        Ok(summary)
    }

    fn find_existing_file(
        transaction: &Transaction<'_>,
        file: &ScannedFile,
    ) -> Result<Option<ExistingTrack>, rusqlite::Error> {
        let file_path = file.path.to_string_lossy();

        transaction
            .query_row(
                "SELECT file_size, file_mtime, missing_since IS NOT NULL
             FROM tracks
             WHERE path = ?1",
                params![file_path.as_ref()],
                |row| {
                    Ok(ExistingTrack {
                        file_size: row.get(0)?,
                        file_mtime: row.get(1)?,
                        is_missing: row.get(2)?,
                    })
                },
            )
            .optional()
    }

    fn process_track(
        scanner: &DirectoryScanner,
        transaction: &Transaction<'_>,
        file: &ScannedFile,
        directory_id: i64,
        now: i64,
    ) -> Result<(), LibraryError> {
        let metadata = DirectoryScanner::extract_metadata(&file.path);

        let track = Track::new(0, file.path.clone(), metadata);

        Self::save_track(
            transaction,
            &track,
            file.file_size,
            file.file_mtime,
            directory_id,
            now,
        )?;

        // Mantém scanner como dependência do processo e permite evolução futura.
        let _ = scanner;

        Ok(())
    }

    fn restore_unchanged_track(
        transaction: &Transaction<'_>,
        file: &ScannedFile,
        directory_id: i64,
        now: i64,
    ) -> Result<(), rusqlite::Error> {
        let file_path = file.path.to_string_lossy();

        transaction.execute(
            "UPDATE tracks
             SET directory_id = ?1,
                 missing_since = NULL,
                 updated_at = ?2
             WHERE path = ?3
               AND (directory_id != ?1 OR missing_since IS NOT NULL)",
            params![directory_id, now, file_path.as_ref()],
        )?;

        Ok(())
    }

    fn save_track(
        transaction: &Transaction<'_>,
        track: &Track,
        file_size: i64,
        file_mtime: i64,
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
                file_size, file_mtime,
                created_at, updated_at
            )
            VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11
            )
            ON CONFLICT(path) DO UPDATE SET
                directory_id = excluded.directory_id,
                artist_id = excluded.artist_id,
                album_id = excluded.album_id,
                title = excluded.title,
                track_number = excluded.track_number,
                genre = excluded.genre,
                duration_ms = excluded.duration_ms,
                file_size = excluded.file_size,
                file_mtime = excluded.file_mtime,
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
                file_size,
                file_mtime,
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

    fn mark_missing_tracks(
        transaction: &Transaction<'_>,
        directory_id: i64,
        now: i64,
        files: &[ScannedFile],
    ) -> Result<(), rusqlite::Error> {
        transaction.execute_batch(
            "
            CREATE TEMP TABLE IF NOT EXISTS scanned_paths (
                path TEXT PRIMARY KEY
            ) WITHOUT ROWID;

            DELETE FROM scanned_paths;
            ",
        )?;

        {
            let mut statement =
                transaction.prepare("INSERT INTO scanned_paths (path) VALUES (?1)")?;

            for file in files {
                let path = file.path.to_string_lossy();
                statement.execute(params![path.as_ref()])?;
            }
        }

        transaction.execute(
            "UPDATE tracks
             SET missing_since = ?1,
                 updated_at = ?1
             WHERE directory_id = ?2
               AND missing_since IS NULL
               AND path NOT IN (
                   SELECT path FROM scanned_paths
               )",
            params![now, directory_id],
        )?;

        Ok(())
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn map_track_metadata(row: &rusqlite::Row<'_>) -> Result<TrackMetadata, rusqlite::Error> {
    let track_number: Option<i64> = row.get(5)?;
    let year: Option<i64> = row.get(6)?;
    let duration_ms: Option<i64> = row.get(8)?;

    Ok(TrackMetadata {
        title: row.get(2)?,
        artist: row.get(3)?,
        album: row.get(4)?,
        track_number: track_number.and_then(|value| u32::try_from(value).ok()),
        year: year.and_then(|value| u32::try_from(value).ok()),
        genre: row.get(7)?,
        duration_ms: duration_ms.and_then(|value| u64::try_from(value).ok()),
    })
}

use rusqlite::{Connection, Result};
use std::path::Path;

use super::schema::{SCHEMA_V1, SEED_V1};

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut connection = Connection::open(path)?;

        connection.execute_batch("PRAGMA foreign_keys = ON;")?;

        Self::initialize(&mut connection)?;

        Ok(Self { connection })
    }

    fn initialize(connection: &mut Connection) -> Result<()> {
        let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;

        match version {
            0 => {
                let has_objects: bool = connection.query_row(
                    "SELECT EXISTS(
                        SELECT 1 FROM sqlite_master
                        WHERE name NOT LIKE 'sqlite_%'
                    )",
                    [],
                    |row| row.get(0),
                )?;

                let transaction = connection.transaction()?;

                if !has_objects {
                    // Base completamente vazia.
                    transaction.execute_batch(SCHEMA_V1)?;
                } else if !Self::is_v1_schema(&transaction)? {
                    // Não reconhecer o esquema: não modificar a base.
                    return Err(rusqlite::Error::InvalidQuery);
                }

                // Seguro executar novamente: INSERT OR IGNORE.
                transaction.execute_batch(SEED_V1)?;

                transaction.execute_batch("PRAGMA user_version = 1;")?;

                transaction.commit()?;
            }

            1 => {
                // Schema já atualizado.
            }

            _ => {
                return Err(rusqlite::Error::InvalidQuery);
            }
        }

        Ok(())
    }

    /// Verifica se a estrutura existente corresponde ao esquema V1.
    fn is_v1_schema(connection: &Connection) -> Result<bool> {
        let required_columns: &[(&str, &[&str])] = &[
            ("watched_directories", &["path", "last_scan", "created_at"]),
            (
                "artists",
                &["name", "name_sort", "created_at", "updated_at"],
            ),
            ("albums", &["title", "artist_id", "total_tracks"]),
            (
                "tracks",
                &[
                    "path",
                    "directory_id",
                    "artist_id",
                    "album_id",
                    "file_mtime",
                    "missing_since",
                ],
            ),
            ("playlists", &["name", "is_smart", "system_key"]),
            (
                "playlist_tracks",
                &["playlist_id", "track_id", "position", "created_at"],
            ),
            ("favorites", &["track_id", "created_at"]),
            ("playback_history", &["track_id", "played_at"]),
        ];

        for (table, columns) in required_columns {
            for column in *columns {
                if !Self::table_has_column(connection, table, column)? {
                    return Ok(false);
                }
            }
        }

        let view_exists: bool = connection.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sqlite_master
                WHERE type = 'view' AND name = 'v_tracks'
            )",
            [],
            |row| row.get(0),
        )?;

        Ok(view_exists)
    }

    /// Verifica se uma tabela possui determinada coluna.
    fn table_has_column(connection: &Connection, table: &str, column: &str) -> Result<bool> {
        let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;

        let columns = statement.query_map([], |row| row.get::<_, String>(1))?;

        for result in columns {
            if result? == column {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_initialization() {
        let database = Database::new(":memory:").expect("A base de dados deve ser inicializada");

        let version: i64 = database
            .connection()
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();

        assert_eq!(version, 1);

        let table_exists: bool = database
            .connection()
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sqlite_master
                    WHERE type = 'table' AND name = 'tracks'
                )",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(table_exists);
    }

    #[test]
    fn test_default_playlists_are_created() {
        let database = Database::new(":memory:").unwrap();

        let count: i64 = database
            .connection()
            .query_row("SELECT COUNT(*) FROM playlists", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 3);
    }

    #[test]
    fn test_foreign_keys_are_enabled() {
        let database = Database::new(":memory:").unwrap();

        let enabled: i64 = database
            .connection()
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();

        assert_eq!(enabled, 1);
    }
}

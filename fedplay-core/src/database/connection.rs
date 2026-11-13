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
                let transaction = connection.transaction()?;

                transaction.execute_batch(SCHEMA_V1)?;
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

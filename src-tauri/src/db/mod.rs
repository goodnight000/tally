pub mod cost_defaults;
pub mod migrations;
pub mod models;
pub mod queries;

use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::path::PathBuf;

use crate::db::migrations::run_migrations;

/// Returns the path to Tally's data directory (~/.tally/)
pub fn tally_data_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".tally")
}

/// Returns the path to the Tally SQLite database
pub fn db_path() -> PathBuf {
    tally_data_dir().join("tally.sqlite")
}

/// Initialize the database: create directory, open connection, run migrations
pub fn init_db() -> Result<Connection, rusqlite::Error> {
    let data_dir = tally_data_dir();
    fs::create_dir_all(&data_dir).expect("Failed to create ~/.tally/ directory");

    let path = db_path();
    let conn = Connection::open(&path)?;

    // Enable WAL mode for better concurrent read performance
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    run_migrations(&conn)?;

    Ok(conn)
}

/// Open a read-only connection for queries (no lock contention with the write connection)
pub fn init_read_db() -> Result<Connection, rusqlite::Error> {
    let path = db_path();
    let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    Ok(conn)
}

/// Open an in-memory database for testing
#[cfg(test)]
pub fn init_test_db() -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    run_migrations(&conn)?;
    Ok(conn)
}

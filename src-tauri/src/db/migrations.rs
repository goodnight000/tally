use rusqlite::Connection;

const MIGRATION_V1: &str = r#"
-- Sync watermarks for incremental updates
CREATE TABLE sync_state (
    source TEXT PRIMARY KEY,
    last_sync_at TEXT NOT NULL,
    last_codex_thread_updated_at INTEGER,
    last_claude_file_offsets TEXT,
    last_codex_db_version INTEGER
);

-- Normalized session data from both tools
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    tool TEXT NOT NULL,
    source TEXT,
    parent_session_id TEXT,
    model TEXT,
    title TEXT,
    start_time TEXT NOT NULL,
    end_time TEXT,
    project_path TEXT,
    project_name TEXT,
    git_branch TEXT,
    git_sha TEXT,
    git_origin_url TEXT,
    cli_version TEXT,
    total_input_tokens INTEGER DEFAULT 0,
    total_output_tokens INTEGER DEFAULT 0,
    total_cache_read_tokens INTEGER DEFAULT 0,
    total_cache_creation_tokens INTEGER DEFAULT 0,
    total_reasoning_tokens INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    FOREIGN KEY (parent_session_id) REFERENCES sessions(id)
);

-- Per-request granular token data
CREATE TABLE requests (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    model TEXT,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    cache_read_tokens INTEGER DEFAULT 0,
    cache_creation_tokens INTEGER DEFAULT 0,
    reasoning_tokens INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    duration_ms INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- User-configured cost rates per model
CREATE TABLE cost_rates (
    model TEXT PRIMARY KEY,
    input_per_million REAL,
    output_per_million REAL,
    cache_read_per_million REAL,
    cache_creation_per_million REAL,
    effective_from TEXT
);

CREATE INDEX idx_sessions_tool ON sessions(tool);
CREATE INDEX idx_sessions_start_time ON sessions(start_time);
CREATE INDEX idx_sessions_project_name ON sessions(project_name);
CREATE INDEX idx_requests_session_id ON requests(session_id);
CREATE INDEX idx_requests_timestamp ON requests(timestamp);
"#;

const MIGRATION_V2: &str = r#"
-- Generic watermark column for any source (replaces source-specific columns)
ALTER TABLE sync_state ADD COLUMN watermark TEXT;

-- Migrate existing Codex watermark data into JSON format
UPDATE sync_state SET watermark = json_object(
    'thread_updated_at', last_codex_thread_updated_at,
    'db_version', last_codex_db_version
) WHERE source = 'codex' AND last_codex_thread_updated_at IS NOT NULL;

-- Migrate existing Claude watermark data into JSON format
UPDATE sync_state SET watermark = json_object(
    'file_offsets', json(last_claude_file_offsets)
) WHERE source = 'claude' AND last_claude_file_offsets IS NOT NULL;

-- User source preferences (enable/disable per source)
CREATE TABLE source_preferences (
    source_id TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

/// All migrations in order. Each entry is (version, sql).
const MIGRATIONS: &[(i32, &str)] = &[(1, MIGRATION_V1), (2, MIGRATION_V2)];

/// Run any pending migrations
pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Ensure schema_version table exists (bootstrap)
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )?;

    for &(version, sql) in MIGRATIONS {
        if version > current_version {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [version],
            )?;
            log::info!("Applied migration v{}", version);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_apply_cleanly() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Verify all tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"schema_version".to_string()));
        assert!(tables.contains(&"sync_state".to_string()));
        assert!(tables.contains(&"sessions".to_string()));
        assert!(tables.contains(&"requests".to_string()));
        assert!(tables.contains(&"cost_rates".to_string()));
        assert!(tables.contains(&"source_preferences".to_string()));
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        // Running again should be a no-op
        run_migrations(&conn).unwrap();

        let version: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn test_v2_migration_adds_watermark_column() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Verify watermark column exists on sync_state
        conn.execute(
            "INSERT INTO sync_state (source, last_sync_at, watermark) VALUES ('test', datetime('now'), '{}')",
            [],
        ).unwrap();

        let wm: String = conn
            .query_row("SELECT watermark FROM sync_state WHERE source = 'test'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(wm, "{}");

        // Verify source_preferences table works
        conn.execute(
            "INSERT INTO source_preferences (source_id, enabled) VALUES ('claude', 1)",
            [],
        ).unwrap();

        let enabled: i32 = conn
            .query_row("SELECT enabled FROM source_preferences WHERE source_id = 'claude'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(enabled, 1);
    }
}

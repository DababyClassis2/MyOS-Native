use rusqlite::{Connection, Result};
use std::path::Path;

pub fn init_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    init_db_in_conn(&conn)?;
    Ok(conn)
}

pub fn init_db_in_conn(conn: &Connection) -> Result<()> {
    // Use PRAGMA query for results to be absolutely safe
    let _ = conn.query_row("PRAGMA foreign_keys = ON;", [], |_| Ok(()));
    let _ = conn.query_row("PRAGMA journal_mode = WAL;", [], |_| Ok(()));
    
    // 1. Settings Table
    let _ = conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key        TEXT    NOT NULL,
            value      TEXT    NOT NULL,
            profile_id TEXT    NOT NULL DEFAULT 'default',
            updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
            PRIMARY KEY (key, profile_id)
        );",
        [],
    );

    // 2. Audit Log Table
    let _ = conn.execute(
        "CREATE TABLE IF NOT EXISTS audit_log (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            ts         INTEGER NOT NULL DEFAULT (unixepoch()),
            module_id  TEXT    NOT NULL,
            action     TEXT    NOT NULL,
            detail     TEXT,
            severity   TEXT    NOT NULL DEFAULT 'INFO',
            profile_id TEXT    NOT NULL DEFAULT 'default'
        );",
        [],
    );

    // 3. Notes Table
    let _ = conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id         TEXT    PRIMARY KEY NOT NULL,
            title      TEXT    NOT NULL,
            body       TEXT    NOT NULL DEFAULT '',
            is_pinned  INTEGER NOT NULL DEFAULT 0,
            profile_id TEXT    NOT NULL DEFAULT 'default',
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            updated_at INTEGER NOT NULL DEFAULT (unixepoch())
        );",
        [],
    );

    // 4. Notes FTS5 Virtual Table
    let _ = conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            title, body, content=notes, content_rowid=rowid
        );",
        [],
    );

    // 5. Workspace State Table
    let _ = conn.execute(
        "CREATE TABLE IF NOT EXISTS workspace_state (
            key   TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );",
        [],
    );

    // 6. Packages Table
    let _ = conn.execute(
        "CREATE TABLE IF NOT EXISTS packages (
            id            TEXT    PRIMARY KEY NOT NULL,
            manifest_json TEXT    NOT NULL,
            enabled       INTEGER NOT NULL DEFAULT 1,
            installed_at  INTEGER NOT NULL DEFAULT (unixepoch())
        );",
        [],
    );

    Ok(())
}

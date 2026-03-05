use rusqlite::{Connection, Result};
use std::fs;
use std::path::PathBuf;

pub fn get_db_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("reqly");

    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }

    path.push("reqly.db");
    path
}

pub fn init_db() -> Result<Connection> {
    let path = get_db_path();
    let conn = Connection::open(path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS urls (
            id INTEGER PRIMARY KEY,
            url TEXT UNIQUE,
            created_at INTEGER
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS requests (
            id INTEGER PRIMARY KEY,
            url_id INTEGER,
            protocol TEXT,
            method TEXT,
            headers TEXT,
            request_body TEXT,
            response_body TEXT,
            status_code INTEGER,
            duration_ms INTEGER,
            created_at INTEGER,
            FOREIGN KEY(url_id) REFERENCES urls(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_requests_url_id ON requests(url_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_requests_created ON requests(created_at DESC)",
        [],
    )?;

    Ok(conn)
}

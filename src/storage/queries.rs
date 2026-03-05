use rusqlite::{Connection, Result, params};

pub fn insert_request(
    conn: &mut Connection,
    protocol: &str,
    url: &str,
    method: &str,
    headers: &str,
    request_body: &str,
    response_body: &str,
    status_code: i64,
    duration_ms: i64,
) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let tx = conn.transaction()?;

    // Insert or ignore URL
    tx.execute(
        "INSERT OR IGNORE INTO urls (url, created_at) VALUES (?1, ?2)",
        params![url, now],
    )?;

    // Get URL ID
    let url_id: i64 = tx.query_row("SELECT id FROM urls WHERE url = ?1", params![url], |row| {
        row.get(0)
    })?;

    // Insert Request
    tx.execute(
        "INSERT INTO requests (
            url_id, protocol, method, headers, request_body, 
            response_body, status_code, duration_ms, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            url_id,
            protocol,
            method,
            headers,
            request_body,
            response_body,
            status_code,
            duration_ms,
            now
        ],
    )?;

    tx.commit()?;
    Ok(())
}

pub fn fetch_recent_urls(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT url FROM requests 
         JOIN urls ON urls.id = requests.url_id 
         ORDER BY requests.created_at DESC 
         LIMIT 50",
    )?;

    let iter = stmt.query_map([], |row| row.get(0))?;
    let mut urls = Vec::new();
    for url in iter {
        urls.push(url?);
    }

    // Fallback exactly to unique URLs if no requests yet but somehow URLs exist
    if urls.is_empty() {
        let mut stmt = conn.prepare("SELECT url FROM urls ORDER BY created_at DESC LIMIT 50")?;
        let iter = stmt.query_map([], |row| row.get(0))?;
        for url in iter {
            urls.push(url?);
        }
    }

    Ok(urls)
}

pub fn fetch_recent_bodies_for_url(conn: &Connection, url: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT request_body 
         FROM requests 
         JOIN urls ON urls.id = requests.url_id 
         WHERE urls.url = ?1 AND request_body != ''
         ORDER BY requests.created_at DESC 
         LIMIT 50",
    )?;

    let iter = stmt.query_map(params![url], |row| row.get(0))?;
    let mut bodies = Vec::new();
    for body in iter {
        bodies.push(body?);
    }
    Ok(bodies)
}

pub fn fetch_recent_bodies_global(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT request_body 
         FROM requests 
         WHERE request_body != ''
         ORDER BY created_at DESC 
         LIMIT 50",
    )?;

    let iter = stmt.query_map([], |row| row.get(0))?;
    let mut bodies = Vec::new();
    for body in iter {
        bodies.push(body?);
    }
    Ok(bodies)
}

pub fn fetch_recent_headers(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT headers 
         FROM requests 
         WHERE headers != ''
         ORDER BY created_at DESC 
         LIMIT 50",
    )?;

    let iter = stmt.query_map([], |row| row.get(0))?;
    let mut headers = Vec::new();
    for header in iter {
        headers.push(header?);
    }
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_in_memory_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE urls (
                id INTEGER PRIMARY KEY,
                url TEXT UNIQUE,
                created_at INTEGER
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE requests (
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
        Ok(conn)
    }

    #[test]
    fn test_insert_and_fetch_history() -> Result<()> {
        let mut conn = setup_in_memory_db()?;

        insert_request(
            &mut conn,
            "HTTP",
            "https://test.com/api",
            "POST",
            "Content-Type: application/json",
            "{\"foo\": \"bar\"}",
            "{\"result\": \"ok\"}",
            200,
            50,
        )?;

        let urls = fetch_recent_urls(&conn)?;
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://test.com/api");

        let bodies = fetch_recent_bodies_for_url(&conn, "https://test.com/api")?;
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0], "{\"foo\": \"bar\"}");

        let global_bodies = fetch_recent_bodies_global(&conn)?;
        assert_eq!(global_bodies.len(), 1);
        assert_eq!(global_bodies[0], "{\"foo\": \"bar\"}");

        let headers = fetch_recent_headers(&conn)?;
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0], "Content-Type: application/json");

        Ok(())
    }
}
